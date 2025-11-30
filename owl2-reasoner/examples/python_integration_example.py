#!/usr/bin/env python3
"""
OWL2 Reasoner Python Integration Example

This example demonstrates how to use the OWL2 reasoner with EPCIS data
through Python bindings for supply chain traceability applications.
"""

import sys
import os
import json
from datetime import datetime
from typing import List, Dict, Any

# Import the Python bindings
try:
    import owl2_reasoner_python as owl2
except ImportError:
    print("Error: owl2_reasoner_python module not found.")
    print("Please build the Python bindings first with: maturin develop")
    sys.exit(1)

def main():
    """Main demonstration of OWL2 reasoner Python integration."""

    print("🐍 **OWL2 Reasoner Python Integration Example**")
    print("=" * 50)

    # 1. Generate sample EPCIS data
    print("\n1️⃣ **Generating Sample EPCIS Data**")
    print("   Creating supply chain events...")

    generator = owl2.create_epcis_generator("small")
    events = generator.generate_events(10)

    print(f"   ✅ Generated {len(events)} EPCIS events")

    # Show event configuration
    config_info = generator.get_config_info()
    print(f"   📊 Configuration: {config_info}")

    # 2. Parse EPCIS data
    print("\n2️⃣ **Parsing EPCIS Data**")
    print("   Converting XML to structured events...")

    # Sample EPCIS XML
    sample_xml = """<?xml version="1.0" encoding="UTF-8"?>
<EPCISDocument xmlns="urn:epcglobal:epcis:xsd:2" schemaVersion="2.0">
    <EPCISBody>
        <EventList>
            <ObjectEvent>
                <eventTime>2023-01-01T10:00:00Z</eventTime>
                <recordTime>2023-01-01T10:00:00Z</recordTime>
                <eventTimeZoneOffset>+00:00</eventTimeZoneOffset>
                <epcList>
                    <epc>urn:epc:id:sgtin:0614141.107346.2023</epc>
                    <epc>urn:epc:id:sgtin:0614141.107346.2024</epc>
                </epcList>
                <action>ADD</action>
                <bizStep>urn:epcglobal:cbv:bizstep:receiving</bizStep>
                <disposition>urn:epcglobal:cbv:disp:in_progress</disposition>
                <readPoint>
                    <id>urn:epc:id:sgln:0614141.00001.0</id>
                </readPoint>
            </ObjectEvent>
        </EventList>
    </EPCISBody>
</EPCISDocument>"""

    try:
        parsed_events = owl2.parse_epcis_xml_string(sample_xml)
        print(f"   ✅ Parsed {len(parsed_events)} events from XML")

        # Show parsed event details
        for i, event in enumerate(parsed_events):
            print(f"   📋 Event {i+1}:")
            print(f"      ID: {event.event_id}")
            print(f"      Type: {event.event_type}")
            print(f"      EPCs: {len(event.epcs)} items")
            print(f"      Action: {event.action}")
    except Exception as e:
        print(f"   ❌ Error parsing XML: {e}")
        # Use generated events instead
        parsed_events = events

    # 3. Load events into OWL2 reasoner
    print("\n3️⃣ **Loading Events into OWL2 Reasoner**")
    print("   Converting EPCIS events to OWL2 ontology...")

    reasoner = owl2.create_reasoner()

    try:
        reasoner.load_epcis_events(parsed_events)
        print("   ✅ Successfully loaded events into reasoner")
    except Exception as e:
        print(f"   ❌ Error loading events: {e}")
        return

    # 4. Perform reasoning operations
    print("\n4️⃣ **Performing Reasoning Operations**")

    # Get ontology statistics
    try:
        stats = reasoner.get_statistics()
        print("   📊 Ontology Statistics:")
        for key, value in stats.items():
            print(f"      {key}: {value}")
    except Exception as e:
        print(f"   ❌ Error getting statistics: {e}")

    # Check consistency
    try:
        is_consistent = reasoner.is_consistent()
        print(f"   ✅ Consistency Check: {'✓ Consistent' if is_consistent else '✗ Inconsistent'}")
    except Exception as e:
        print(f"   ❌ Error checking consistency: {e}")

    # Validate OWL2 profiles
    try:
        el_valid = reasoner.validate_el_profile()
        ql_valid = reasoner.validate_ql_profile()
        rl_valid = reasoner.validate_rl_profile()

        print("   📈 OWL2 Profile Validation:")
        print(f"      EL Profile: {'✓ Valid' if el_valid else '✗ Invalid'}")
        print(f"      QL Profile: {'✓ Valid' if ql_valid else '✗ Invalid'}")
        print(f"      RL Profile: {'✓ Valid' if rl_valid else '✗ Invalid'}")
    except Exception as e:
        print(f"   ❌ Error validating profiles: {e}")

    # 5. Advanced analysis
    print("\n5️⃣ **Advanced EPCIS Analysis**")

    # Use parser for additional analysis
    parser = owl2.PyEPCISParser()

    # Extract all EPCs
    all_epcs = parser.extract_all_epcs(parsed_events)
    print(f"   🏷️  Unique EPCs: {len(all_epcs)}")

    # Extract events by type
    events_by_type = parser.extract_events_by_type(parsed_events)
    print("   📊 Events by Type:")
    for event_type, count in events_by_type.items():
        print(f"      {event_type}: {count}")

    # 6. Generate comprehensive report
    print("\n6️⃣ **Generating Analysis Report**")

    report = {
        "analysis_timestamp": datetime.now().isoformat(),
        "total_events": len(parsed_events),
        "unique_epcs": len(all_epcs),
        "ontology_stats": reasoner.get_statistics(),
        "consistency_status": reasoner.is_consistent(),
        "profile_validation": {
            "el": reasoner.validate_el_profile(),
            "ql": reasoner.validate_ql_profile(),
            "rl": reasoner.validate_rl_profile()
        },
        "event_distribution": events_by_type,
        "sample_events": [event.to_dict() for event in parsed_events[:3]]
    }

    # Save report to file
    report_file = "epcis_analysis_report.json"
    with open(report_file, 'w') as f:
        json.dump(report, f, indent=2)

    print(f"   📄 Analysis report saved to: {report_file}")
    print(f"   📊 Report includes: {len(report)} data sections")

    # 7. Demonstration complete
    print("\n🎉 **Python Integration Demo Complete**")
    print("=" * 50)
    print("✅ **Key Capabilities Demonstrated:**")
    print("   • EPCIS XML parsing and processing")
    print("   • OWL2 ontology creation from EPCIS events")
    print("   • Consistency checking and profile validation")
    print("   • Statistical analysis and reporting")
    print("   • Python-native API integration")

    print("\n🔧 **Next Steps for Integration:**")
    print("   • Connect to real EPCIS data sources")
    print("   • Implement custom reasoning rules")
    print("   • Add visualization and dashboard")
    print("   • Integrate with supply chain systems")

    print(f"\n📁 **Generated Files:**")
    print(f"   • {report_file} - Comprehensive analysis report")

    return report

def advanced_example():
    """Advanced example showing complex EPCIS workflows."""

    print("\n🔬 **Advanced EPCIS Workflow Example**")
    print("=" * 40)

    # Create larger dataset
    generator = owl2.create_epcis_generator("medium")
    events = generator.generate_events(100)

    print(f"📊 Generated {len(events)} events for advanced analysis")

    # Multiple analysis passes
    reasoner = owl2.create_reasoner()

    # Batch processing
    batch_size = 25
    for i in range(0, len(events), batch_size):
        batch = events[i:i+batch_size]
        try:
            reasoner.load_epcis_events(batch)
            print(f"   📦 Processed batch {i//batch_size + 1}: {len(batch)} events")
        except Exception as e:
            print(f"   ❌ Batch processing error: {e}")

    # Advanced analytics
    try:
        stats = reasoner.get_statistics()
        print(f"   📈 Final ontology contains {stats.get('individuals', 0)} individuals")

        is_consistent = reasoner.is_consistent()
        print(f"   ✅ Large-scale consistency: {'✓ Pass' if is_consistent else '✗ Fail'}")

        # Profile validation
        profiles = {
            "EL": reasoner.validate_el_profile(),
            "QL": reasoner.validate_ql_profile(),
            "RL": reasoner.validate_rl_profile()
        }

        valid_profiles = [name for name, valid in profiles.items() if valid]
        print(f"   🎯 Valid OWL2 profiles: {', '.join(valid_profiles)}")

    except Exception as e:
        print(f"   ❌ Advanced analytics error: {e}")

if __name__ == "__main__":
    try:
        # Run main example
        report = main()

        # Run advanced example
        advanced_example()

        print("\n🚀 **Integration Ready for Production!**")
        print("The OWL2 reasoner is now accessible through Python bindings.")

    except Exception as e:
        print(f"❌ Error in example: {e}")
        sys.exit(1)