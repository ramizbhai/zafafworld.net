import os
import sys
import json
import click
from rich.console import Console
from rich.table import Table

# Add current directory to path to support imports when running as a module/script
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from security_analyzer.analyzer import SecurityAnalyzer
from security_analyzer.report_generator import generate_report

console = Console()

@click.group()
def cli():
    """ZafafWorld AI-Powered Threat Modeling & Compliance Tool."""
    pass

@cli.command()
@click.argument('target', type=click.Path(exists=True))
@click.option('--output', '-o', default='report.html', help='Path to output the HTML report.')
@click.option('--feedback', '-f', default='human_feedback.json', help='Path to the human feedback JSON file.')
def analyze(target, output, feedback):
    """Run security threat modeling & compliance audit on the specified target."""
    console.print(f"[bold green]Starting security analysis on: {target}[/bold green]")
    try:
        analyzer = SecurityAnalyzer(feedback_file=feedback)
        result = analyzer.analyze_path(target)
        
        # Serialize the Pydantic model to dict
        result_dict = result.model_dump()
        
        # Generate the interactive HTML report
        generate_report(result_dict, output)
        
        # Print summary
        console.print("\n[bold green]Analysis Completed Successfully![/bold green]")
        console.print(f"  - [bold]Threats Found:[/] {len(result.threats)}")
        console.print(f"  - [bold]Operational Issues:[/] {len(result.operational_issues)}")
        console.print(f"  - [bold]Compliance Gaps:[/] {len(result.regulatory_gaps)}")
        console.print(f"  - [bold]Report Location:[/] {os.path.abspath(output)}")
        
    except Exception as e:
        console.print(f"[bold red]Error during analysis:[/] {str(e)}")
        sys.exit(1)

@cli.command()
@click.argument('target', type=click.Path(exists=True))
@click.option('--feedback', '-f', default='human_feedback.json', help='Path to the human feedback JSON file.')
def evaluate(target, feedback):
    """Evaluate current model output against human feedback and calculate alignment metrics."""
    console.print(f"[bold blue]Evaluating model alignment using feedback from: {feedback}[/bold blue]")
    try:
        analyzer = SecurityAnalyzer(feedback_file=feedback)
        if not os.path.exists(feedback) or os.path.getsize(feedback) == 0:
            console.print("[yellow]No feedback history found. Please add a rule first using the Feedback Studio in the HTML report.[/yellow]")
            return

        # Perform fresh analysis to check if it respects/aligns with feedback
        result = analyzer.analyze_path(target)
        metrics = analyzer.feedback_manager.evaluate_performance(result.model_dump())

        # Render Table
        table = Table(title="Model Alignment Metrics")
        table.add_column("Metric", style="cyan")
        table.add_column("Value", style="magenta")

        table.add_row("Total Feedback Rules Checked", str(metrics["total_feedback_rules_checked"]))
        table.add_row("Mismatches Found", str(metrics["mismatches_found"]))
        table.add_row("Alignment Score", f"{metrics['alignment_percentage']}%")
        table.add_row("Avg Severity/Score Deviation", str(metrics["average_deviation_severity_or_score"]))

        console.print(table)
        
        if metrics["alignment_percentage"] >= 90:
            console.print("[bold green]Excellent! Model alignment is highly consistent with human feedback.[/bold green]")
        elif metrics["alignment_percentage"] >= 70:
            console.print("[bold yellow]Good. Model alignment is acceptable but has room for improvement.[/bold yellow]")
        else:
            console.print("[bold red]Warning: Low model alignment. Consider providing more detailed feedback or adjusting system prompts.[/bold red]")
            
    except Exception as e:
        console.print(f"[bold red]Error during evaluation:[/] {str(e)}")
        sys.exit(1)

if __name__ == '__main__':
    cli()
