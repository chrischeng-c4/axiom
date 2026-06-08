"""
Agent Evaluation - LLM Judge with Prompt Templates

Demonstrates how to use different prompt engineering techniques (templates)
with the LLM-as-judge evaluation system:

1. Basic: Simple evaluation prompt
2. Few-Shot: Calibration examples for consistency
3. Chain-of-Thought: Step-by-step reasoning for explainability
4. Self-Consistency: Multiple sampling for reliability

Requirements:
- Set OPENAI_API_KEY environment variable
- Run from project root: `uv run --env-file=.env python examples/agent_eval_llm_judge_templates.py`
"""

import asyncio
from cclab.nova import Agent, OpenAI, Anthropic
from cclab.nova_eval import AgentEvaluator, AgentTestCase, LLMJudgeConfig


async def main():
    print("=" * 80)
    print("LLM Judge with Prompt Templates - Comparison")
    print("=" * 80)

    # Create test case
    test_case = AgentTestCase(
        id="test-001",
        name="Capital question with context",
        input="What is the capital of France? Please explain briefly.",
        expected_output="Paris",
        quality_criteria=[
            {"name": "accuracy", "description": "Is the answer factually correct?", "weight": 2.0},
            {"name": "relevance", "description": "Does it answer the question?", "weight": 1.5},
            {"name": "clarity", "description": "Is the explanation clear?", "weight": 1.0},
        ],
    )

    # Mock agent response
    async def agent_fn(input_text: str) -> dict:
        return {
            "content": "Paris is the capital of France. It has been the capital since 987 CE.",
            "usage": {"prompt_tokens": 20, "completion_tokens": 15, "total_tokens": 35},
            "model": "gpt-4o-mini",
        }

    # Test different templates
    templates = [
        ("llm_judge_basic", "Basic evaluation (default)"),
        ("llm_judge_few_shot", "Few-shot learning (improved consistency)"),
        ("llm_judge_cot", "Chain-of-thought (explainable)"),
        ("llm_judge_self_consistency", "Self-consistency (high reliability)"),
    ]

    print("\n" + "=" * 80)
    print("Testing Different Prompt Templates")
    print("=" * 80)

    for template_name, description in templates:
        print(f"\n{description}")
        print("-" * 80)

        # Create evaluator with specific template
        llm_judge_config = LLMJudgeConfig(
            model="gpt-4o-mini",
            provider="openai",
            temperature=0.0 if "consistency" not in template_name else 0.7,
            template_name=template_name,
        )

        evaluator = AgentEvaluator(
            test_cases=[test_case],
            enable_llm_judge=True,
            llm_judge_config=llm_judge_config,
        )

        try:
            report = await evaluator.evaluate(agent_fn)

            # Print results
            result = report.results[0]
            print(f"✓ Overall Score: {result.quality_scores.overall_score:.2f}")
            print(f"  Scores: {result.quality_scores.aws}")
            if result.quality_scores.feedback:
                print(f"  Feedback: {result.quality_scores.feedback}")

        except Exception as e:
            print(f"✗ Error: {e}")

    print("\n" + "=" * 80)
    print("When to Use Each Template")
    print("=" * 80)

    print("""
    1. Basic Template (llm_judge_basic)
       - Use case: General evaluation, fast iteration
       - Temperature: 0.0 (deterministic)
       - Best for: Initial development, quick feedback

    2. Few-Shot Template (llm_judge_few_shot)
       - Use case: Improved consistency across evaluations
       - Temperature: 0.0 (deterministic)
       - Best for: When eval_consistency < 0.8
       - Includes 3 calibration examples

    3. Chain-of-Thought Template (llm_judge_cot)
       - Use case: Explainable evaluation with reasoning
       - Temperature: 0.0 (deterministic)
       - Best for: Debugging, understanding failures, accuracy < 0.85
       - Returns step-by-step reasoning in response

    4. Self-Consistency Template (llm_judge_self_consistency)
       - Use case: High-reliability evaluation
       - Temperature: 0.7 (sampling)
       - Best for: Critical evaluation, false positive rate > 5%
       - Use with multiple samples (n=5) + majority voting
       - Higher cost but more reliable
    """)

    print("\n" + "=" * 80)
    print("Advanced Usage: Custom Templates")
    print("=" * 80)

    print("""
    You can create custom templates by adding YAML files to templates/llm_judge/:

    ```yaml
    name: my_custom_template
    version: "1.0.0"
    description: "Custom evaluation template"

    system_role: "You are an expert evaluator..."

    sections:
      - title: "Input"
        content: "{{input}}"

      - title: "Expected Output"
        content: "{{expected}}"
        optional: true
        condition: "has_expected"

      - title: "Actual Output"
        content: "{{actual}}"

      - title: "Evaluation Criteria"
        content: |
          {{criteria}}

      - title: "Instructions"
        content: |
          Your custom evaluation instructions here.

          Respond in JSON format:
          ```json
          {
            "scores": {
              {{criteria_keys}}
            },
            "feedback": "Your feedback"
          }
          ```

    metadata:
      technique: "custom"
      temperature: "0.0"
      use_case: "specialized_evaluation"
    ```

    Then use it:
    ```python
    llm_judge_config = LLMJudgeConfig(
        model="gpt-4o-mini",
        template_name="my_custom_template",
        template_version="1.0.0",  # Optional: specify version
    )
    ```
    """)


if __name__ == "__main__":
    asyncio.run(main())
