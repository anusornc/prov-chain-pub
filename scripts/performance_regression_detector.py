#!/usr/bin/env python3
"""
Performance Regression Detection System for ProvChain-Org

This script automatically detects performance regressions by:
- Comparing current performance metrics against historical baselines
- Identifying significant performance degradations
- Alerting on potential regressions
- Generating performance trend reports
"""

import json
import sys
import os
import argparse
import statistics
import smtplib
from datetime import datetime, timedelta
from email.mime.text import MimeText
from email.mime.multipart import MimeMultipart
from pathlib import Path
from typing import Dict, List, Any, Optional, Tuple
import numpy as np
from scipy import stats
import matplotlib.pyplot as plt
import seaborn as sns

# Configuration
BASELINE_FILE = "metrics/performance_baseline.json"
CURRENT_METRICS_FILE = "metrics/current_metrics.json"
HISTORICAL_DATA_DIR = "metrics/historical"
REGRESSION_THRESHOLD = 15.0  # 15% degradation threshold
CONFIDENCE_LEVEL = 0.95
MIN_SAMPLE_SIZE = 10

class PerformanceRegressionDetector:
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.baseline_metrics = self.load_baseline()
        self.current_metrics = self.load_current_metrics()
        self.historical_metrics = self.load_historical_metrics()

    def load_baseline(self) -> Dict[str, Any]:
        """Load baseline performance metrics"""
        try:
            with open(BASELINE_FILE, 'r') as f:
                return json.load(f)
        except FileNotFoundError:
            print(f"Warning: Baseline file {BASELINE_FILE} not found. Using empty baseline.")
            return {}

    def load_current_metrics(self) -> Dict[str, Any]:
        """Load current performance metrics"""
        try:
            with open(CURRENT_METRICS_FILE, 'r') as f:
                return json.load(f)
        except FileNotFoundError:
            print(f"Error: Current metrics file {CURRENT_METRICS_FILE} not found.")
            sys.exit(1)

    def load_historical_metrics(self) -> List[Dict[str, Any]]:
        """Load historical performance metrics"""
        historical = []
        try:
            for file_path in Path(HISTORICAL_DATA_DIR).glob("*.json"):
                with open(file_path, 'r') as f:
                    data = json.load(f)
                    historical.append(data)
        except FileNotFoundError:
            print(f"Warning: Historical data directory {HISTORICAL_DATA_DIR} not found.")

        return sorted(historical, key=lambda x: x.get('timestamp', ''))

    def detect_regressions(self) -> List[Dict[str, Any]]:
        """Detect performance regressions"""
        regressions = []

        # Compare current metrics with baseline
        for metric_name, baseline_value in self.baseline_metrics.items():
            if metric_name in self.current_metrics:
                current_value = self.current_metrics[metric_name]

                if isinstance(baseline_value, (int, float)) and isinstance(current_value, (int, float)):
                    regression = self._check_metric_regression(
                        metric_name, baseline_value, current_value
                    )
                    if regression:
                        regressions.append(regression)

        # Check trends in historical data
        if len(self.historical_metrics) >= MIN_SAMPLE_SIZE:
            trend_regressions = self._detect_trend_regressions()
            regressions.extend(trend_regressions)

        return regressions

    def _check_metric_regression(self, metric_name: str, baseline: float, current: float) -> Optional[Dict[str, Any]]:
        """Check if a single metric shows regression"""
        # For response times and memory usage, higher is worse
        # For throughput and success rates, lower is worse
        is_higher_worse = any(keyword in metric_name.lower() for keyword in
                           ['time', 'latency', 'memory', 'cpu', 'disk'])

        if is_higher_worse:
            change_percent = ((current - baseline) / baseline) * 100
            if change_percent > REGRESSION_THRESHOLD:
                return {
                    'metric': metric_name,
                    'type': 'baseline_comparison',
                    'baseline': baseline,
                    'current': current,
                    'change_percent': change_percent,
                    'severity': self._calculate_severity(change_percent),
                    'description': f"{metric_name} degraded by {change_percent:.1f}% from baseline"
                }
        else:
            change_percent = ((baseline - current) / baseline) * 100
            if change_percent > REGRESSION_THRESHOLD:
                return {
                    'metric': metric_name,
                    'type': 'baseline_comparison',
                    'baseline': baseline,
                    'current': current,
                    'change_percent': change_percent,
                    'severity': self._calculate_severity(change_percent),
                    'description': f"{metric_name} degraded by {change_percent:.1f}% from baseline"
                }

        return None

    def _detect_trend_regressions(self) -> List[Dict[str, Any]]:
        """Detect regressions based on historical trends"""
        regressions = []

        # Group metrics by type
        metric_types = {}
        for data_point in self.historical_metrics:
            for metric_name, value in data_point.items():
                if metric_name != 'timestamp' and isinstance(value, (int, float)):
                    if metric_name not in metric_types:
                        metric_types[metric_name] = []
                    metric_types[metric_name].append(value)

        # Analyze trends
        for metric_name, values in metric_types.items():
            if len(values) >= MIN_SAMPLE_SIZE:
                trend_regression = self._analyze_trend(metric_name, values)
                if trend_regression:
                    regressions.append(trend_regression)

        return regressions

    def _analyze_trend(self, metric_name: str, values: List[float]) -> Optional[Dict[str, Any]]:
        """Analyze trend for a specific metric"""
        # Calculate linear regression
        x = np.arange(len(values))
        slope, intercept, r_value, p_value, std_err = stats.linregress(x, values)

        # Check if trend is statistically significant
        if p_value < (1 - CONFIDENCE_LEVEL):
            is_higher_worse = any(keyword in metric_name.lower() for keyword in
                               ['time', 'latency', 'memory', 'cpu', 'disk'])

            # Positive slope for metrics where higher is worse, negative for others
            if (is_higher_worse and slope > 0) or (not is_higher_worse and slope < 0):
                # Calculate total change over the period
                total_change = slope * len(values)
                baseline_avg = statistics.mean(values[:5])  # First 5 measurements
                change_percent = (abs(total_change) / baseline_avg) * 100

                if change_percent > REGRESSION_THRESHOLD:
                    return {
                        'metric': metric_name,
                        'type': 'trend_analysis',
                        'slope': slope,
                        'r_squared': r_value ** 2,
                        'p_value': p_value,
                        'change_percent': change_percent,
                        'severity': self._calculate_severity(change_percent),
                        'description': f"{metric_name} shows significant degrading trend (p={p_value:.4f})"
                    }

        return None

    def _calculate_severity(self, change_percent: float) -> str:
        """Calculate severity level based on change percentage"""
        if change_percent >= 50:
            return 'critical'
        elif change_percent >= 30:
            return 'high'
        elif change_percent >= 20:
            return 'medium'
        else:
            return 'low'

    def generate_report(self, regressions: List[Dict[str, Any]]) -> str:
        """Generate performance regression report"""
        report = []
        report.append("# Performance Regression Report")
        report.append(f"Generated: {datetime.now().isoformat()}")
        report.append(f"Regressions Detected: {len(regressions)}")
        report.append("")

        if not regressions:
            report.append("✅ No performance regressions detected!")
        else:
            # Group by severity
            severity_groups = {}
            for regression in regressions:
                severity = regression['severity']
                if severity not in severity_groups:
                    severity_groups[severity] = []
                severity_groups[severity].append(regression)

            for severity in ['critical', 'high', 'medium', 'low']:
                if severity in severity_groups:
                    report.append(f"## {severity.upper()} SEVERITY ({len(severity_groups[severity])})")
                    for regression in severity_groups[severity]:
                        report.append(f"- **{regression['metric']}**: {regression['description']}")
                        if 'baseline' in regression:
                            report.append(f"  - Baseline: {regression['baseline']:.2f}")
                            report.append(f"  - Current: {regression['current']:.2f}")
                            report.append(f"  - Change: {regression['change_percent']:.1f}%")
                        report.append("")

        # Add performance summary
        report.append("## Performance Summary")
        report.append(self._generate_performance_summary())

        return "\n".join(report)

    def _generate_performance_summary(self) -> str:
        """Generate performance summary section"""
        summary = []

        # Overall performance score
        score = self._calculate_performance_score()
        summary.append(f"**Overall Performance Score**: {score}/100")

        # Key metrics overview
        if self.current_metrics:
            summary.append("\n### Key Metrics:")
            key_metrics = ['response_time', 'throughput', 'cpu_usage', 'memory_usage']
            for metric in key_metrics:
                for key, value in self.current_metrics.items():
                    if metric in key.lower() and isinstance(value, (int, float)):
                        summary.append(f"- {key}: {value:.2f}")
                        break

        return "\n".join(summary)

    def _calculate_performance_score(self) -> int:
        """Calculate overall performance score (0-100)"""
        if not self.current_metrics or not self.baseline_metrics:
            return 50  # Default score

        scores = []
        for metric_name, baseline_value in self.baseline_metrics.items():
            if metric_name in self.current_metrics:
                current_value = self.current_metrics[metric_name]
                if isinstance(baseline_value, (int, float)) and isinstance(current_value, (int, float)):
                    # Calculate score for this metric
                    is_higher_worse = any(keyword in metric_name.lower() for keyword in
                                       ['time', 'latency', 'memory', 'cpu', 'disk'])

                    if is_higher_worse:
                        ratio = baseline_value / current_value
                    else:
                        ratio = current_value / baseline_value

                    # Clamp ratio and convert to 0-100 scale
                    score = min(100, max(0, ratio * 100))
                    scores.append(score)

        return int(statistics.mean(scores)) if scores else 50

    def generate_visualizations(self, output_dir: str = "reports") -> None:
        """Generate performance visualization charts"""
        os.makedirs(output_dir, exist_ok=True)

        # Prepare data for visualization
        timestamps = []
        metrics_data = {}

        for data_point in self.historical_metrics:
            if 'timestamp' in data_point:
                timestamps.append(data_point['timestamp'])

                for key, value in data_point.items():
                    if key != 'timestamp' and isinstance(value, (int, float)):
                        if key not in metrics_data:
                            metrics_data[key] = []
                        metrics_data[key].append(value)

        # Generate charts for key metrics
        key_metrics = ['response_time', 'throughput', 'cpu_usage', 'memory_usage']

        for metric in key_metrics:
            matching_keys = [key for key in metrics_data.keys() if metric in key.lower()]

            for key in matching_keys[:1]:  # Take first matching key for each metric
                if len(metrics_data[key]) >= 2:
                    plt.figure(figsize=(12, 6))

                    # Plot historical data
                    plt.plot(range(len(metrics_data[key])), metrics_data[key],
                           label='Historical', marker='o', alpha=0.7)

                    # Add current value if available
                    if key in self.current_metrics:
                        current_value = self.current_metrics[key]
                        plt.plot(len(metrics_data[key]), current_value,
                               'ro', markersize=10, label='Current')

                    # Add baseline if available
                    if key in self.baseline_metrics:
                        baseline_value = self.baseline_metrics[key]
                        plt.axhline(y=baseline_value, color='r', linestyle='--',
                                   label='Baseline', alpha=0.7)

                    plt.title(f'{key.replace("_", " ").title()} Over Time')
                    plt.xlabel('Time (measurements)')
                    plt.ylabel(key.replace("_", " ").title())
                    plt.legend()
                    plt.grid(True, alpha=0.3)

                    # Save chart
                    chart_path = os.path.join(output_dir, f'{key}_trend.png')
                    plt.savefig(chart_path, dpi=300, bbox_inches='tight')
                    plt.close()

                    print(f"Generated chart: {chart_path}")

    def send_alert(self, regressions: List[Dict[str, Any]]) -> None:
        """Send alert for detected regressions"""
        if not regressions:
            return

        # Only alert for high and critical severity regressions
        alert_regressions = [r for r in regressions if r['severity'] in ['high', 'critical']]

        if not alert_regressions:
            return

        subject = f"🚨 Performance Regression Alert - {len(alert_regressions)} Issues Detected"

        body = f"""
Performance regressions detected in ProvChain-Org:

Critical Issues: {len([r for r in alert_regressions if r['severity'] == 'critical'])}
High Issues: {len([r for r in alert_regressions if r['severity'] == 'high'])}

Details:
"""

        for regression in alert_regressions:
            body += f"\n- {regression['description']}"

        body += f"\n\nReport generated: {datetime.now().isoformat()}"

        # Send email if configured
        if 'email' in self.config:
            self._send_email(subject, body, self.config['email'])

        # Send Slack notification if configured
        if 'slack_webhook' in self.config:
            self._send_slack_notification(subject, alert_regressions, self.config['slack_webhook'])

    def _send_email(self, subject: str, body: str, email_config: Dict[str, str]) -> None:
        """Send email alert"""
        try:
            msg = MimeMultipart()
            msg['From'] = email_config['from']
            msg['To'] = email_config['to']
            msg['Subject'] = subject
            msg.attach(MimeText(body, 'plain'))

            with smtplib.SMTP(email_config['smtp_server'], email_config['smtp_port']) as server:
                server.starttls()
                server.login(email_config['username'], email_config['password'])
                server.send_message(msg)

            print(f"Email alert sent to {email_config['to']}")
        except Exception as e:
            print(f"Failed to send email alert: {e}")

    def _send_slack_notification(self, subject: str, regressions: List[Dict[str, Any]], webhook_url: str) -> None:
        """Send Slack notification"""
        try:
            import requests

            # Count by severity
            critical_count = len([r for r in regressions if r['severity'] == 'critical'])
            high_count = len([r for r in regressions if r['severity'] == 'high'])

            message = {
                "text": subject,
                "attachments": [
                    {
                        "color": "danger",
                        "fields": [
                            {
                                "title": "Summary",
                                "value": f"Critical: {critical_count}, High: {high_count}",
                                "short": True
                            },
                            {
                                "title": "Time",
                                "value": datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
                                "short": True
                            }
                        ]
                    }
                ]
            }

            # Add details for critical regressions
            critical_regressions = [r for r in regressions if r['severity'] == 'critical']
            for regression in critical_regressions[:5]:  # Limit to first 5
                message["attachments"].append({
                    "color": "danger",
                    "text": f"• {regression['description']}"
                })

            requests.post(webhook_url, json=message)
            print("Slack notification sent")
        except Exception as e:
            print(f"Failed to send Slack notification: {e}")

    def update_baseline(self) -> None:
        """Update baseline with current metrics"""
        try:
            with open(BASELINE_FILE, 'w') as f:
                json.dump(self.current_metrics, f, indent=2)
            print(f"Baseline updated with {len(self.current_metrics)} metrics")
        except Exception as e:
            print(f"Failed to update baseline: {e}")


def load_config(config_file: str = "performance_config.json") -> Dict[str, Any]:
    """Load configuration from file"""
    default_config = {
        "regression_threshold": 15.0,
        "confidence_level": 0.95,
        "min_sample_size": 10
    }

    try:
        with open(config_file, 'r') as f:
            config = json.load(f)
            return {**default_config, **config}
    except FileNotFoundError:
        print(f"Config file {config_file} not found, using defaults")
        return default_config


def main():
    parser = argparse.ArgumentParser(description="Performance Regression Detector")
    parser.add_argument("--config", default="performance_config.json",
                       help="Configuration file path")
    parser.add_argument("--update-baseline", action="store_true",
                       help="Update baseline with current metrics")
    parser.add_argument("--generate-report", action="store_true",
                       help="Generate performance report only")
    parser.add_argument("--generate-charts", action="store_true",
                       help="Generate performance visualization charts")
    parser.add_argument("--output-dir", default="reports",
                       help="Output directory for reports and charts")

    args = parser.parse_args()

    # Load configuration
    config = load_config(args.config)

    # Initialize detector
    detector = PerformanceRegressionDetector(config)

    # Detect regressions
    regressions = detector.detect_regressions()

    # Generate report
    report = detector.generate_report(regressions)

    # Save report
    report_file = os.path.join(args.output_dir, f"performance_report_{datetime.now().strftime('%Y%m%d_%H%M%S')}.md")
    os.makedirs(args.output_dir, exist_ok=True)

    with open(report_file, 'w') as f:
        f.write(report)

    print(f"Performance report saved to: {report_file}")
    print(f"\n{report}")

    # Generate charts if requested
    if args.generate_charts:
        detector.generate_visualizations(args.output_dir)

    # Send alerts if regressions detected
    if regressions:
        detector.send_alert(regressions)

    # Update baseline if requested
    if args.update_baseline:
        detector.update_baseline()

    # Exit with appropriate code
    critical_regressions = [r for r in regressions if r['severity'] == 'critical']
    if critical_regressions:
        print(f"\n❌ {len(critical_regressions)} critical regressions detected!")
        sys.exit(1)
    elif regressions:
        print(f"\n⚠️  {len(regressions)} regressions detected (no critical issues)")
        sys.exit(2)
    else:
        print("\n✅ No performance regressions detected!")
        sys.exit(0)


if __name__ == "__main__":
    main()