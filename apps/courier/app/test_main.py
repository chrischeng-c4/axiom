import unittest
from unittest.mock import AsyncMock, MagicMock, patch
from fastapi.testclient import TestClient
import pytest
from app.main import app

client = TestClient(app)

class TestCourierApp(unittest.TestCase):
    def test_health_checks(self):
        resp = client.get("/healthz")
        self.assertEqual(resp.status_code, 200)
        self.assertEqual(resp.json(), {"status": "ok"})

        resp = client.get("/readyz")
        self.assertEqual(resp.status_code, 200)
        self.assertEqual(resp.json(), {"status": "ok"})

    def test_get_issues_disallowed_repo(self):
        resp = client.get("/issues?repo=some/other-repo")
        self.assertEqual(resp.status_code, 403)
        self.assertIn("not in COURIER_ALLOWED_REPOS", resp.json()["detail"])

    @patch("app.main.forward_to_github", new_callable=AsyncMock)
    def test_get_issues_single_view(self, mock_forward):
        # Mock success response from GitHub
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"number": 123, "title": "Test Issue"}
        mock_forward.return_value = mock_response

        resp = client.get("/issues?repo=chrischeng-c4/axiom&number=123")
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

        resp = client.get("/issues?repo=chrischeng-c4/axiom&q=query&state=closed")
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

        resp = client.post("/issues", json=payload)
        self.assertEqual(resp.status_code, 200)
        results = resp.json()
        self.assertEqual(len(results), 2)
        self.assertEqual(results[0]["status"], 201)
        self.assertEqual(results[0]["body"]["title"], "created")
        self.assertEqual(results[1]["status"], 201)
        self.assertEqual(results[1]["body"]["body"], "commented")

        # Verify call arguments
        self.assertEqual(mock_forward.call_count, 3)
