"""
Simple Agent Example

Demonstrates basic agent usage with OpenAI provider.
"""

import asyncio
import os
from cclab.nova import Agent, OpenAI


async def main():
    # Get API key from environment
    api_key = os.getenv("OPENAI_API_KEY")
    if not api_key:
        print("Error: OPENAI_API_KEY environment variable not set")
        return

    # Create OpenAI provider (using gpt-4o-mini - fastest and cheapest)
    llm = OpenAI(api_key=api_key, model="gpt-4o-mini")
    print(f"✓ Created OpenAI provider: {llm.provider_name}")
    print(f"  Supported models: {llm.supported_models}")

    # Create agent
    agent = Agent(
        name="assistant",
        llm=llm,
        system_prompt="You are a helpful assistant. Be concise and friendly.",
        max_turns=10,
    )
    print(f"\n✓ Created agent: {agent.name}")

    # Get agent configuration
    config = agent.get_config()
    print(f"  Configuration:")
    print(f"    - Agent ID: {config['agent_id']}")
    print(f"    - System Prompt: {config['system_prompt']}")
    print(f"    - Max Turns: {config['max_turns']}")
    print(f"    - Tool Timeout: {config['tool_timeout_secs']}s")

    # Example 1: Simple question
    print("\n" + "="*60)
    print("Example 1: Simple Question")
    print("="*60)

    question1 = "What's the capital of France?"
    print(f"\nUser: {question1}")

    response1 = await agent.run(question1, temperature=0.7)
    print(f"\nAssistant: {response1['content']}")
    print(f"\nMetadata:")
    print(f"  - Model: {response1['model']}")
    print(f"  - Finish Reason: {response1['finish_reason']}")
    print(f"  - Tokens: {response1['usage']['total_tokens']} "
          f"(prompt: {response1['usage']['prompt_tokens']}, "
          f"completion: {response1['usage']['completion_tokens']})")

    # Example 2: Complex question
    print("\n" + "="*60)
    print("Example 2: Complex Question")
    print("="*60)

    question2 = "Explain the difference between async and sync programming in Python in 2 sentences."
    print(f"\nUser: {question2}")

    response2 = await agent.run(question2, temperature=0.5, max_tokens=150)
    print(f"\nAssistant: {response2['content']}")
    print(f"\nMetadata:")
    print(f"  - Model: {response2['model']}")
    print(f"  - Finish Reason: {response2['finish_reason']}")
    print(f"  - Tokens: {response2['usage']['total_tokens']}")

    # Example 3: Different model
    print("\n" + "="*60)
    print("Example 3: Using Different Model")
    print("="*60)

    question3 = "What's 25 * 17?"
    print(f"\nUser: {question3}")

    response3 = await agent.run(question3, model="gpt-3.5-turbo", temperature=0.0)
    print(f"\nAssistant: {response3['content']}")
    print(f"\nMetadata:")
    print(f"  - Model: {response3['model']}")
    print(f"  - Tokens: {response3['usage']['total_tokens']}")

    print("\n" + "="*60)
    print("✓ All examples completed successfully!")
    print("="*60)


if __name__ == "__main__":
    asyncio.run(main())
