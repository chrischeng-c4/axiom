# SPEC-MANAGED: apps/courier/tech-design/semantic/app/test_main.md#text-source-unit
# CODEGEN-BEGIN
# HANDWRITE-BEGIN
import os
import unittest
from unittest.mock import AsyncMock, MagicMock, patch
from fastapi.testclient import TestClient
from app.main import app

client = TestClient(app)

@patch.dict(os.environ, {
    "COURIER_ACCEPTED_TOKENS": "test-token",
    "COURIER_GITHUB_TOKEN": "mock-github-token"
})
class TestCourierApp(unittest.TestCase):
    def setUp(self):
        self.headers = {"Authorization": "Bearer test-token"}

    def test_health_checks(self):
        resp = client.get("/healthz")
        self.assertEqual(resp.status_code, 200)
        self.assertEqual(resp.json(), {"status": "ok"})

        resp = client.get("/readyz")
        self.assertEqual(resp.status_code, 200)
        self.assertEqual(resp.json(), {"status": "ok"})

    def test_auth_failures(self):
        # No header
        resp = client.get("/issues?repo=chrischeng-c4/axiom")
        self.assertEqual(resp.status_code, 401)
        
        # Wrong token
        resp = client.get("/issues?repo=chrischeng-c4/axiom", headers={"Authorization": "Bearer wrong"})
        self.assertEqual(resp.status_code, 401)

        # No header on POST
        resp = client.post("/issues", json={})
        self.assertEqual(resp.status_code, 401)

    def test_get_issues_disallowed_repo(self):
        resp = client.get("/issues?repo=some/other-repo", headers=self.headers)
        self.assertEqual(resp.status_code, 403)
        self.assertIn("not in COURIER_ALLOWED_REPOS", resp.json()["detail"])

    @patch("app.main.forward_to_github", new_callable=AsyncMock)
    def test_get_issues_single_view(self, mock_forward):
        # Mock success response from GitHub
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"number": 123, "title": "Test Issue"}
        mock_forward.return_value = mock_response

        resp = client.get("/issues?repo=chrischeng-c4/axiom&number=123", headers=self.headers)
        self.assertEqual(resp.status_code, 200)
        self.assertEqual(resp.json(), {"number": 123, "title": "Test Issue"})
        mock_forward.assert_called_once()
        # Verify path
        self.assertEqual(mock_forward.call_args[0][2], "/repos/chrischeng-c4/axiom/issues/123")

    @patch("app.main.forward_to_github", new_callable=AsyncMock)
    def test_get_issues_search(self, mock_forward):
        # Mock success search response
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"total_count": 1, "items": [{"number": 123}]}
        mock_forward.return_value = mock_response

        resp = client.get("/issues?repo=chrischeng-c4/axiom&q=query&state=closed", headers=self.headers)
        self.assertEqual(resp.status_code, 200)
        self.assertEqual(resp.json()["total_count"], 1)
        mock_forward.assert_called_once()
        # Verify query params
        params = mock_forward.call_args[1]["params"]
        self.assertEqual(params["q"], "repo:chrischeng-c4/axiom is:issue state:closed query")

    @patch("app.main.forward_to_github", new_callable=AsyncMock)
    def test_post_issues_batch(self, mock_forward):
        # Mock success responses for mixed batch
        mock_resp_create = MagicMock()
        mock_resp_create.status_code = 201
        mock_resp_create.json.return_value = {"id": 1, "number": 1, "title": "created"}

        mock_resp_comment = MagicMock()
        mock_resp_comment.status_code = 201
        mock_resp_comment.json.return_value = {"id": 2, "body": "commented"}

        # Return mock_resp_create for first call, mock_resp_comment for second call
        # Note: comment operation performs 2 calls (PATCH state=open, then POST comment)
        # So we have: call 1 (create), call 2 (reopen), call 3 (comment)
        mock_resp_reopen = MagicMock()
        mock_resp_reopen.status_code = 200

        mock_forward.side_effect = [mock_resp_create, mock_resp_reopen, mock_resp_comment]

        payload = {
            "repo": "chrischeng-c4/axiom",
            "ops": [
                {
                    "op": "create",
                    "title": "New issue",
                    "body": "body text",
                    "labels": ["bug"]
                },
                {
                    "op": "comment",
                    "number": 123,
                    "body": "my comment"
                }
            ]
        }

        resp = client.post("/issues", json=payload, headers=self.headers)
        self.assertEqual(resp.status_code, 200)
        results = resp.json()
        self.assertEqual(len(results), 2)
        self.assertEqual(results[0]["status"], 201)
        self.assertEqual(results[0]["body"]["title"], "created")
        self.assertEqual(results[1]["status"], 201)
        self.assertEqual(results[1]["body"]["body"], "commented")

        # Verify call arguments
        self.assertEqual(mock_forward.call_count, 3)

    def test_contract_validation(self):
        import json
        from app.main import BatchMutationRequest

        # Load contract_fixture.json relative to this file
        fixture_path = os.path.join(os.path.dirname(__file__), "contract_fixture.json")
        with open(fixture_path) as f:
            fixture = json.load(f)

        for case_name, case_data in fixture.items():
            expected = case_data["expected_server_payload"]
            parsed = BatchMutationRequest.model_validate(expected)
            self.assertEqual(parsed.repo, expected["repo"])
            self.assertEqual(len(parsed.ops), len(expected["ops"]))
            for p_op, e_op in zip(parsed.ops, expected["ops"]):
                self.assertEqual(p_op.op.value, e_op["op"])
                self.assertEqual(p_op.title, e_op.get("title"))
                self.assertEqual(p_op.body, e_op.get("body"))
                self.assertEqual(p_op.labels, e_op.get("labels"))
                self.assertEqual(p_op.number, e_op.get("number"))
                self.assertEqual(p_op.state, e_op.get("state"))
# HANDWRITE-END
# CODEGEN-END
