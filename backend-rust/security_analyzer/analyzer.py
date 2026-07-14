import os
import json
from typing import List, Dict, Any
from security_analyzer.config import AnalyzerConfig, AnalysisResult
from security_analyzer.parser import get_context_from_path
from security_analyzer.feedback import FeedbackManager
from security_analyzer.llm import DeepSeekClient

SYSTEM_ANALYSIS_PROMPT = """You are an elite principal security engineer and cloud architect.
Your task is to conduct a thorough security threat modeling, risk assessment, and regulatory compliance audit on the provided code, configurations, or design documentation.

Be extremely rigorous. Search for vulnerabilities, design flaws, misconfigurations, and compliance gaps.
You must analyze the inputs using the following frameworks:
1. STRIDE (Spoofing, Tampering, Repudiation, Information Disclosure, Denial of Service, Elevation of Privilege)
2. PASTA (Process for Attack Simulation and Threat Analysis)
3. LINDDUN (Linkability, Identifiability, Non-repudiation, Detectability, Disclosure of information, Unawareness, Non-compliance)
4. OWASP Top 10 (Web/API Security risks)

Additionally, identify:
- Operational risks: Single points of failure, unsafe dependencies, lack of redundancy, and misconfigurations.
- Regulatory gaps: Violations against GDPR, HIPAA, SOC2, and PCI-DSS.

When outputting:
- Ensure all IDs (e.g. THR-xxx, OPS-xxx, REG-xxx) are unique.
- Map threats to specific CWE IDs (Common Weakness Enumeration) and MITRE ATT&CK techniques where possible.
- Provide a clear, step-by-step 'justification' of the severity rating (Critical, High, Medium, Low) and the 0.0-10.0 risk score.
"""

SYSTEM_CONSOLIDATION_PROMPT = """You are a principal security architect.
You have been given a set of security reports generated from analyzing different parts of the same system.
Your job is to:
1. De-duplicate identical or highly overlapping threats, operational issues, and regulatory gaps.
2. Merge related findings into a single, comprehensive item, combining their details into a unified description.
3. Ensure consistent ID numbering (THR-001, THR-002, ..., OPS-001, ..., REG-001, ...).
4. Verify and normalize severity levels and risk scores.
5. Provide the output in a clean, fully populated structure.
"""

class SecurityAnalyzer:
    def __init__(self, feedback_file: str = "human_feedback.json"):
        self.config = AnalyzerConfig()
        self.client = DeepSeekClient(self.config)
        self.feedback_manager = FeedbackManager(feedback_file)

    def chunk_text(self, text: str, max_chars: int = 15000) -> List[str]:
        """Splits the input text into manageable chunks by line boundaries."""
        lines = text.split("\n")
        chunks = []
        current_chunk = []
        current_length = 0
        
        for line in lines:
            if current_length + len(line) + 1 > max_chars:
                chunks.append("\n".join(current_chunk))
                current_chunk = [line]
                current_length = len(line)
            else:
                current_chunk.append(line)
                current_length += len(line) + 1
        
        if current_chunk:
            chunks.append("\n".join(current_chunk))
            
        return chunks

    def analyze_path(self, path: str) -> AnalysisResult:
        """Loads, chunks, analyzes, and consolidates the targets at the specified path."""
        print(f"[*] Reading and parsing target: {path}...")
        raw_context = get_context_from_path(path)
        
        chunks = self.chunk_text(raw_context)
        print(f"[*] Split content into {len(chunks)} chunk(s) for analysis.")
        
        partial_results = []
        feedback_prompt = self.feedback_manager.get_feedback_context_prompt()

        for idx, chunk in enumerate(chunks):
            print(f"[*] Analyzing chunk {idx + 1}/{len(chunks)}...")
            user_prompt = f"{feedback_prompt}\n\nAnalyze the following system context:\n\n{chunk}"
            
            result = self.client.generate_structured_response(
                system_prompt=SYSTEM_ANALYSIS_PROMPT,
                user_prompt=user_prompt,
                response_model=AnalysisResult
            )
            partial_results.append(result)

        if len(partial_results) == 1:
            # Only one chunk, no consolidation necessary, but run it through a cleaning step
            # to make sure everything conforms perfectly.
            final_result = partial_results[0]
        else:
            print("[*] Consolidating and de-duplicating multiple analysis results...")
            final_result = self.consolidate_results(partial_results)

        # Append metadata
        final_result.metadata = {
            "target_path": os.path.abspath(path),
            "total_chunks": len(chunks),
            "threats_found": len(final_result.threats),
            "operational_issues_found": len(final_result.operational_issues),
            "regulatory_gaps_found": len(final_result.regulatory_gaps)
        }
        
        return final_result

    def consolidate_results(self, results: List[AnalysisResult]) -> AnalysisResult:
        """Converts multiple AnalysisResult objects into one consolidated set."""
        combined_input = {
            "threats": [],
            "operational_issues": [],
            "regulatory_gaps": []
        }
        
        for r in results:
            for t in r.threats:
                combined_input["threats"].append(t.model_dump())
            for o in r.operational_issues:
                combined_input["operational_issues"].append(o.model_dump())
            for g in r.regulatory_gaps:
                combined_input["regulatory_gaps"].append(g.model_dump())
                
        user_prompt = f"Please consolidate and de-duplicate the following items:\n\n{json.dumps(combined_input, indent=2)}"
        
        consolidated = self.client.generate_structured_response(
            system_prompt=SYSTEM_CONSOLIDATION_PROMPT,
            user_prompt=user_prompt,
            response_model=AnalysisResult
        )
        return consolidated
