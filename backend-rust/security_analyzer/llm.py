import os
import json
import time
from typing import Type, TypeVar, Dict, Any
from pydantic import BaseModel
from openai import OpenAI
from security_analyzer.config import AnalyzerConfig

T = TypeVar('T', bound=BaseModel)

class DeepSeekClient:
    def __init__(self, config: AnalyzerConfig):
        self.config = config
        if not self.config.api_key:
            raise ValueError("No API key found. Please set DEEPSEEK_API_KEY or OPENAI_API_KEY environment variables.")
        self.client = OpenAI(
            api_key=self.config.api_key,
            base_url=self.config.base_url
        )

    def generate_structured_response(self, system_prompt: str, user_prompt: str, response_model: Type[T], max_retries: int = 3) -> T:
        """
        Calls the DeepSeek API with a system prompt and user prompt, enforcing
        the output schema defined by response_model (Pydantic model).
        Includes basic retry logic.
        """
        # Ensure we request a JSON response
        response_format = {"type": "json_object"}
        
        # We append instructions to the system prompt to enforce the exact schema
        schema_json = json.dumps(response_model.model_json_schema(), indent=2)
        full_system_prompt = f"{system_prompt}\n\nYou must return a JSON object that strictly conforms to the following JSON Schema:\n{schema_json}"
        
        for attempt in range(max_retries):
            try:
                response = self.client.chat.completions.create(
                    model=self.config.model_name,
                    messages=[
                        {"role": "system", "content": full_system_prompt},
                        {"role": "user", "content": user_prompt}
                    ],
                    response_format=response_format,
                    temperature=self.config.temperature,
                    max_tokens=self.config.max_tokens
                )
                
                content = response.choices[0].message.content
                # Parse and validate the response against the Pydantic model
                validated_data = response_model.model_validate_json(content)
                return validated_data
            except Exception as e:
                print(f"Error on attempt {attempt + 1}: {str(e)}")
                if attempt == max_retries - 1:
                    raise e
                time.sleep(2 ** attempt)  # Exponential backoff
        
        raise RuntimeError("Failed to generate response after maximum retries.")
