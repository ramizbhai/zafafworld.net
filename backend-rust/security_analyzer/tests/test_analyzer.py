import unittest
from unittest.mock import MagicMock, patch
import os
import json
import tempfile
from security_analyzer.config import AnalyzerConfig, AnalysisResult, Threat, OperationalIssue, RegulatoryGap
from security_analyzer.parser import clean_html, get_context_from_path
from security_analyzer.feedback import FeedbackManager
from security_analyzer.report_generator import generate_report

class TestSecurityAnalyzer(unittest.TestCase):

    def test_clean_html(self):
        html_input = "<html><head><style>body {color: red;}</style></head><body><h1>Hello World</h1><script>console.log('hi');</script></body></html>"
        expected = "Hello World"
        self.assertEqual(clean_html(html_input).strip(), expected)

    def test_feedback_manager(self):
        with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as temp_file:
            temp_path = temp_file.name
        
        try:
            manager = FeedbackManager(feedback_file=temp_path)
            # Add new feedback
            manager.add_feedback("Test Threat", "severity", "High", "Low", "Mitigated by WAF")
            
            # Load back
            new_manager = FeedbackManager(feedback_file=temp_path)
            self.assertEqual(len(new_manager.feedback_list), 1)
            self.assertEqual(new_manager.feedback_list[0]["threat_title"], "Test Threat")
            self.assertEqual(new_manager.feedback_list[0]["corrected_value"], "Low")

            # Check prompt generation
            prompt = new_manager.get_feedback_context_prompt()
            self.assertIn("Test Threat", prompt)
            self.assertIn("Mitigated by WAF", prompt)

            # Test evaluation
            mock_results = {
                "threats": [
                    {
                        "title": "Test Threat",
                        "severity": "High", # different from corrected "Low"
                        "id": "THR-001"
                    }
                ],
                "operational_issues": [],
                "regulatory_gaps": []
            }
            metrics = new_manager.evaluate_performance(mock_results)
            self.assertEqual(metrics["total_feedback_rules_checked"], 1)
            self.assertEqual(metrics["mismatches_found"], 1)
            self.assertEqual(metrics["alignment_percentage"], 0.0)

        finally:
            if os.path.exists(temp_path):
                os.remove(temp_path)

    def test_report_generation(self):
        with tempfile.NamedTemporaryFile(suffix=".html", delete=False) as temp_file:
            temp_path = temp_file.name

        try:
            sample_data = {
                "threats": [
                    {
                        "id": "THR-001",
                        "title": "SQL Injection",
                        "description": "Exploiting input fields",
                        "framework": "STRIDE",
                        "category": "Tampering",
                        "severity": "Critical",
                        "score": 9.5,
                        "cwe_id": "CWE-89",
                        "mitre_attack_id": "T1190",
                        "affected_component": "Database",
                        "remediation": "Use parameterized queries",
                        "justification": "Direct SQL execution on user input."
                    }
                ],
                "operational_issues": [],
                "regulatory_gaps": [],
                "metadata": {
                    "target_path": "/fake/path",
                    "total_chunks": 1
                }
            }
            generate_report(sample_data, temp_path)
            self.assertTrue(os.path.exists(temp_path))
            with open(temp_path, 'r', encoding='utf-8') as f:
                content = f.read()
                self.assertIn("SQL Injection", content)
                self.assertIn("CWE-89", content)
        finally:
            if os.path.exists(temp_path):
                os.remove(temp_path)

if __name__ == "__main__":
    unittest.main()
