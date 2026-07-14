import os
import json
from typing import List, Dict, Any

class FeedbackManager:
    def __init__(self, feedback_file: str = "human_feedback.json"):
        self.feedback_file = feedback_file
        self.feedback_list = self.load_feedback()

    def load_feedback(self) -> List[Dict[str, Any]]:
        if not os.path.exists(self.feedback_file) or os.path.getsize(self.feedback_file) == 0:
            return []
        try:
            with open(self.feedback_file, 'r', encoding='utf-8') as f:
                data = json.load(f)
                return data if isinstance(data, list) else []
        except Exception as e:
            print(f"Warning: Failed to load feedback file: {e}")
            return []

    def save_feedback(self):
        try:
            with open(self.feedback_file, 'w', encoding='utf-8') as f:
                json.dump(self.feedback_list, f, indent=2)
        except Exception as e:
            print(f"Error saving feedback: {e}")

    def add_feedback(self, threat_title: str, field: str, original_val: Any, corrected_val: Any, reason: str):
        # Update if exists, or append
        updated = False
        for item in self.feedback_list:
            if item.get("threat_title") == threat_title and item.get("field") == field:
                item["original_value"] = original_val
                item["corrected_value"] = corrected_val
                item["reason"] = reason
                updated = True
                break
        if not updated:
            self.feedback_list.append({
                "threat_title": threat_title,
                "field": field,
                "original_value": original_val,
                "corrected_value": corrected_val,
                "reason": reason
            })
        self.save_feedback()

    def get_feedback_context_prompt(self) -> str:
        """Generates a text block describing previous corrections for in-context learning."""
        if not self.feedback_list:
            return ""
        
        prompt = "\n=== IMPORTANT: PAST HUMAN CORRECTIONS (LEARN & ALIGN WITH THESE RULES) ===\n"
        prompt += "The user has previously corrected the following AI-generated findings. Ensure future evaluations respect these rules:\n"
        for idx, item in enumerate(self.feedback_list, 1):
            prompt += f"{idx}. For threat/issue related to '{item['threat_title']}', the '{item['field']}' should be changed from '{item['original_value']}' to '{item['corrected_value']}' because: {item['reason']}\n"
        prompt += "Apply this style of reasoning, context, and risk tolerance across all new/existing findings.\n\n"
        return prompt

    def evaluate_performance(self, current_results: Dict[str, Any]) -> Dict[str, Any]:
        """Compares current results against historical feedback to find deviations."""
        total_checks = 0
        mismatches = 0
        severity_map = {"Low": 1, "Medium": 2, "High": 3, "Critical": 4}
        score_diffs = []

        all_items = current_results.get("threats", []) + current_results.get("operational_issues", [])

        for feedback in self.feedback_list:
            target_title = feedback.get("threat_title")
            target_field = feedback.get("field")
            corrected_val = feedback.get("corrected_value")

            # Find matching item in current results
            matched_item = None
            for item in all_items:
                if item.get("title", "").lower() == target_title.lower():
                    matched_item = item
                    break
            
            if matched_item:
                total_checks += 1
                current_val = matched_item.get(target_field)
                if str(current_val).lower() != str(corrected_val).lower():
                    mismatches += 1
                
                # If evaluating score, compute numeric delta
                if target_field == "score":
                    try:
                        score_diffs.append(abs(float(current_val) - float(corrected_val)))
                    except (ValueError, TypeError):
                        pass
                # If evaluating severity, compute delta
                elif target_field == "severity":
                    try:
                        curr_num = severity_map.get(current_val, 0)
                        corr_num = severity_map.get(corrected_val, 0)
                        if curr_num > 0 and corr_num > 0:
                            score_diffs.append(abs(curr_num - corr_num))
                    except Exception:
                        pass

        alignment_score = ((total_checks - mismatches) / total_checks * 100) if total_checks > 0 else 100.0
        avg_score_deviation = (sum(score_diffs) / len(score_diffs)) if score_diffs else 0.0

        return {
            "total_feedback_rules_checked": total_checks,
            "mismatches_found": mismatches,
            "alignment_percentage": round(alignment_score, 2),
            "average_deviation_severity_or_score": round(avg_score_deviation, 2)
        }
