# ZafafWorld AI-Powered Threat Modeling & Compliance Tool

An enterprise-grade, automated security threat modeling and regulatory compliance analysis tool integrated with **DeepSeek-V3** (via OpenAI-compatible API). 

This tool scans source code, configurations, and documentation (including PDFs and HTML/Markdown), identifies threats based on major industry frameworks, and produces a fully interactive, self-contained HTML report. 

It also includes a **self-improving feedback loop** that adjusts the AI's risk assessments based on a historical database of human-supplied overrides/rules.

---

## Features
1. **Multi-Framework Threat Modeling**: Map vulnerabilities, design risks, and threats against:
   - **STRIDE** (Spoofing, Tampering, Repudiation, Information Disclosure, Denial of Service, Elevation of Privilege)
   - **PASTA** (Process for Attack Simulation and Threat Analysis)
   - **LINDDUN** (Privacy threat modeling)
   - **OWASP Top 10** (Web & API vulnerabilities)
2. **Regulatory Compliance Audit**: Auto-detect compliance gaps and propose remediations for:
   - **GDPR**
   - **HIPAA**
   - **SOC2**
   - **PCI-DSS**
3. **Operational Risk Assessment**: Detect single points of failure, unhandled error paths, and misconfigurations.
4. **Self-Improving Feedback Loop**:
   - Save human corrections of AI-generated threat/risk assessments to a local `human_feedback.json` file.
   - Automatically inject past corrections as in-context learning rules into the prompt of subsequent analysis runs.
   - Run a built-in evaluator to calculate the alignment (accuracy, mean absolute error/deviation) of new runs against historical human decisions.
5. **Interactive HTML Report**: A single-file, zero-dependency dashboard containing the full analysis report:
   - Interactive KPI cards and SVG-based charts.
   - Searchable, filterable, and paginated tables of threats, operational issues, and compliance gaps.
   - **Interactive Topology Graph**: A visual map displaying components and their associated threats. Clicking on a component filters the threat list.
   - **Feedback Studio**: A UI for creating and exporting corrective overrides to copy/paste into your `human_feedback.json` file.

---

## Installation & Setup

1. **Prerequisites**: Python 3.8+
2. **Install Dependencies**:
   ```bash
   pip install -r requirements.txt
   ```
3. **Configure API Key**:
   Set the `DEEPSEEK_API_KEY` (or `OPENAI_API_KEY`) environment variable. Optionally set custom endpoint or model name.
   ```bash
   export DEEPSEEK_API_KEY="your-api-key"
   # Optional configurations:
   # export DEEPSEEK_API_BASE="https://api.deepseek.com/v1"
   # export DEEPSEEK_MODEL="deepseek-chat"
   ```

---

## CLI Usage

### 1. Run Security Analysis
Scan a file or an entire directory and generate an interactive report:
```bash
python3 -m security_analyzer.cli analyze /path/to/target --output report.html
```

### 2. Run Evaluation (Feedback Loop Validation)
Check the alignment rate of the current model against the historical `human_feedback.json` file:
```bash
python3 -m security_analyzer.cli evaluate /path/to/target --feedback human_feedback.json
```

---

## How the Feedback Loop Works
1. Run the analysis on a target repository or file.
2. Open the generated `report.html` in your browser.
3. Navigate to the **Feedback** tab.
4. If the AI incorrectly rated a threat (e.g. gave `High` to a minor risk), select the threat, input the corrected value, and provide the justification.
5. Click **Generate Override**. Copy the JSON snippet.
6. Append this snippet into your local `human_feedback.json` file (create the file if it doesn't exist, e.g. as a JSON array `[...]`).
7. Run the next analysis! The analyzer will automatically inject the rule to other runs and adhere to the desired risk tolerance.
