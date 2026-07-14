import os
from typing import List, Dict, Any, Optional
from pydantic import BaseModel, Field

class Threat(BaseModel):
    id: str = Field(..., description="Unique alphanumeric identifier (e.g. THR-001)")
    title: str = Field(..., description="Short, descriptive title of the threat")
    description: str = Field(..., description="Detailed description of the threat scenario and impact")
    framework: str = Field(..., description="The framework to which this threat belongs: 'STRIDE', 'PASTA', 'LINDDUN', or 'OWASP'")
    category: str = Field(..., description="Category matching the framework (e.g. Spoofing, Denial of Service, Linkability, Injection, atc.)")
    severity: str = Field(..., description="Severity level: 'Critical', 'High', 'Medium', or 'Low'")
    score: float = Field(..., description="CVSS-style score from 0.0 to 10.0")
    cwe_id: Optional[str] = Field(None, description="Associated CWE ID if applicable (e.g. CWE-79)")
    mitre_attack_id: Optional[str] = Field(None, description="Associated MITRE ATT&CK technique ID if applicable (e.g. T1059)")
    affected_component: str = Field(..., description="The system component, module, or service affected by this threat")
    remediation: str = Field(..., description="Actionable mitigation steps or remediation strategies")
    justification: str = Field(..., description="Step-by-step reasoning for assigning this severity and score")

class OperationalIssue(BaseModel):
    id: str = Field(..., description="Unique alphanumeric identifier (e.g. OPS-001)")
    title: str = Field(..., description="Title of the operational issue")
    description: str = Field(..., description="Detailed explanation of the operational risk, misconfiguration, or single point of failure")
    severity: str = Field(..., description="Severity level: 'High', 'Medium', or 'Low'")
    component: str = Field(..., description="The architectural component or configuration file affected")
    impact: str = Field(..., description="Business or operational impact if this issue is triggered")
    recommendation: str = Field(..., description="Actionable advice to resolve the issue")

class RegulatoryGap(BaseModel):
    id: str = Field(..., description="Unique alphanumeric identifier (e.g. REG-001)")
    regulation: str = Field(..., description="The regulation or standard breached (e.g. GDPR, HIPAA, SOC2, PCI-DSS)")
    clause: str = Field(..., description="Specific clause or control ID (e.g. Article 32, CC6.1)")
    description: str = Field(..., description="Description of the requirement and the identified gap")
    gap_analysis: str = Field(..., description="How the current system fails to meet this requirement")
    remediation: str = Field(..., description="Steps required to achieve compliance")

class AnalysisResult(BaseModel):
    threats: List[Threat] = Field(default_factory=list, description="List of identified security threats")
    operational_issues: List[OperationalIssue] = Field(default_factory=list, description="List of identified operational issues/misconfigurations")
    regulatory_gaps: List[RegulatoryGap] = Field(default_factory=list, description="List of identified regulatory compliance gaps")
    metadata: Dict[str, Any] = Field(default_factory=dict, description="Metadata about the analysis session (timestamp, files analyzed, etc.)")

class AnalyzerConfig:
    def __init__(self):
        self.api_key = os.getenv("DEEPSEEK_API_KEY") or os.getenv("OPENAI_API_KEY")
        self.base_url = os.getenv("DEEPSEEK_API_BASE") or "https://api.deepseek.com/v1"
        self.model_name = os.getenv("DEEPSEEK_MODEL") or "deepseek-chat"
        self.max_tokens = int(os.getenv("DEEPSEEK_MAX_TOKENS", "4000"))
        self.temperature = float(os.getenv("DEEPSEEK_TEMPERATURE", "0.1"))
