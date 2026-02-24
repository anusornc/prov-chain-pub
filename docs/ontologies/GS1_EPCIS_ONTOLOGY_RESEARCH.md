# GS1 EPCIS Ontology Research Report

## Executive Summary

This document provides a comprehensive analysis of GS1 EPCIS (Electronic Product Code Information Services) ontologies available on GitHub that can be integrated into the ProvChain UHT (Ultra-High Temperature) Milk Supply Chain Demo.

**Research Date:** February 23, 2026  
**Researcher:** AI Agent  
**Ontologies Found:** 4 major ontologies + SHACL validation rules

---

## 1. Overview of GS1 EPCIS Standard

GS1 EPCIS is the global standard for supply chain traceability, enabling the capture and sharing of interoperable information about product status, location, movement, and chain of custody.

### Key Features of EPCIS 2.0:
- **JSON/JSON-LD Support:** Modern web-friendly formats
- **RDF/Linked Data:** Full semantic web compatibility
- **Sensor Data:** IoT integration for temperature, humidity, etc.
- **REST API:** Standardized web service interfaces
- **SHACL Validation:** Machine-readable data quality rules

---

## 2. Ontologies Identified

### 2.1 EPCIS Core Ontology

**Source:** https://github.com/gs1/EPCIS  
**File:** `Ontology/EPCIS.ttl`  
**Namespace:** `https://ref.gs1.org/epcis/`  
**Size:** ~60KB  
**Version:** 2.0

**Purpose:**  
Defines the core classes and properties for EPCIS events including ObjectEvent, AggregationEvent, AssociationEvent, TransactionEvent, and TransformationEvent.

**Key Classes for UHT Demo:**

| Class | Description | UHT Use Case |
|-------|-------------|--------------|
| `epcis:ObjectEvent` | Events about physical objects | Milk collection, quality tests |
| `epcis:TransformationEvent` | Input/output transformation | UHT processing (raw milk → UHT milk) |
| `epcis:AggregationEvent` | Packing/unpacking events | Cases, pallets, shipments |
| `epcis:SensorElement` | Sensor data container | Temperature monitoring |
| `epcis:SensorReport` | Individual sensor readings | Cold chain verification |
| `epcis:BizTransaction` | Business documents | Purchase orders, BOLs |

**Key Properties:**
- `epcis:eventTime` - When the event occurred
- `epcis:readPoint` - Where the event was captured
- `epcis:bizLocation` - Business location
- `epcis:bizStep` - Business process step (from CBV)
- `epcis:disposition` - Business condition
- `epcis:action` - ADD, OBSERVE, DELETE
- `epcis:epcList` - Instance-level identifiers (EPCs)
- `epcis:quantityList` - Class-level quantities

---

### 2.2 Core Business Vocabulary (CBV) Ontology

**Source:** https://github.com/gs1/EPCIS  
**File:** `Ontology/CBV.ttl`  
**Namespace:** `https://ref.gs1.org/cbv/`  
**Size:** ~69KB  
**Version:** 2.0

**Purpose:**  
Provides standardized code lists and values for EPCIS events, ensuring interoperability across different systems and organizations.


#### Business Steps (cbv:BizStep)

| Code | Description | UHT Use Case |
|------|-------------|--------------|
| `cbv:BizStep-commissioning` | First association of ID with object | Farm milk collection |
| `cbv:BizStep-creating_class_instance` | Producing class-level quantity | UHT batch production |
| `cbv:BizStep-transporting` | Moving goods via vehicle | Milk transport, distribution |
| `cbv:BizStep-receiving` | Taking into inventory | DC receipt, retail receiving |
| `cbv:BizStep-storing` | Moving into storage | Cold storage |
| `cbv:BizStep-shipping` | Overall outbound process | Shipment to DC/retail |
| `cbv:BizStep-inspecting` | Quality review | Lab testing, quality control |
| `cbv:BizStep-sampling` | Destructive quality testing | Bacteria testing |
| `cbv:BizStep-packing` | Putting into container | Case packing |
| `cbv:BizStep-unpacking` | Removing from container | Retail shelf stocking |
| `cbv:BizStep-retail_selling` | Transfer to customer | Consumer purchase |
| `cbv:BizStep-sensor_reporting` | Sensor data capture | Temperature logging |

#### Dispositions (cbv:Disp)

| Code | Description | UHT Use Case |
|------|-------------|--------------|
| `cbv:Disp-active` | Just commissioned | New batch created |
| `cbv:Disp-in_transit` | Between locations | Transport status |
| `cbv:Disp-in_progress` | Processing | UHT treatment ongoing |
| `cbv:Disp-conformant` | Passed inspection | Quality test passed |
| `cbv:Disp-non_conformant` | Failed inspection | Quality test failed |
| `cbv:Disp-expired` | Past expiration | Expired product |
| `cbv:Disp-damaged` | Impaired product | Damaged in transit |
| `cbv:Disp-container_closed` | Sealed container | Sealed truck/container |

---

### 2.3 EPCIS SHACL Validation Shapes

**Source:** https://github.com/gs1/EPCIS  
**File:** `Ontology/EPCIS-SHACL.ttl`  
**Size:** ~36KB

**Purpose:**  
Provides machine-readable validation rules for EPCIS data using SHACL (Shapes Constraint Language). Ensures data quality and standard compliance.

**Key Validation Features:**

| Shape | Purpose | Validation |
|-------|---------|------------|
| `epcis:ObjectEventShape` | Validate ObjectEvent | Required fields, forbidden fields |
| `epcis:TransformationEventShape` | Validate TransformationEvent | Input/output constraints |
| `epcis:AggregationEventShape` | Validate AggregationEvent | Parent/child constraints |
| `epcis:EventTimeShape` | Validate timestamps | ISO 8601 format, timezone |
| `epcis:ActionShape` | Validate action field | ADD, OBSERVE, DELETE only |
| `epcis:SensorElementShape` | Validate sensor data | Measurement types, values |

---

### 2.4 GS1 Web Vocabulary

**Source:** https://github.com/gs1/WebVoc  
**File:** `v1.11/gs1Voc_v1_11.ttl`  
**Namespace:** `https://gs1.org/voc/`  
**Size:** ~759KB  
**Version:** 1.11

**Purpose:**  
Comprehensive product and organization vocabulary extending schema.org. Provides detailed product attributes, especially for food and beverage products.

**Key Classes for UHT Demo:**

| Class | Description | Properties |
|-------|-------------|------------|
| `gs1:FoodBeverageTobaccoProduct` | Food/beverage product | `ingredientStatement`, `nutritionalInfo` |
| `gs1:MilkButterCreamYogurtCheeseEggsSubstitutes` | Dairy category | Category-specific attributes |
| `gs1:Beverage` | Potable liquid | `alcoholicBeverageSubcategory` |
| `gs1:Product` | Generic product | `gtin`, `brand`, `productName` |
| `gs1:Place` | Location | `address`, `geoCoordinates` |
| `gs1:Organization` | Business entity | `gln`, `organizationName` |
| `gs1:CertificationDetails` | Certifications | `certificationStandard`, `certificationValue` |
| `gs1:NutritionalInfo` | Nutrition facts | `energyPerServing`, `fatPerServing` |
| `gs1:AllergenDetails` | Allergen info | `allergenType`, `levelOfContainment` |


---

## 3. Files Downloaded

| File | Source | Size | Purpose |
|------|--------|------|---------|
| `epcis.ttl` | GS1/EPCIS | 60KB | Core EPCIS ontology |
| `cbv.ttl` | GS1/EPCIS | 69KB | Business vocabulary |
| `epcis-shacl.ttl` | GS1/EPCIS | 36KB | Validation rules |
| `gs1-web-vocab.ttl` | GS1/WebVoc | 759KB | Product vocabulary |

**Location:** `docs/ontologies/gs1_epcis/`

---

## 4. Recommended Integration for UHT Demo

### Namespace Prefixes

```turtle
@prefix epcis: <https://ref.gs1.org/epcis/> .
@prefix cbv: <https://ref.gs1.org/cbv/> .
@prefix gs1: <https://gs1.org/voc/> .
@prefix uht: <https://provchain.org/uht/> .
```

### UHT-Specific Extensions

Create custom UHT ontology extending GS1 standards:

```turtle
uht:UHTProcessingEvent a rdfs:Class ;
    rdfs:subClassOf epcis:TransformationEvent ;
    rdfs:label "UHT Processing Event" .

uht:pasteurizationTemperature a rdf:Property ;
    rdfs:domain uht:UHTProcessingEvent ;
    rdfs:range xsd:decimal ;
    rdfs:label "Pasteurization Temperature (C)" .

uht:holdingTime a rdf:Property ;
    rdfs:domain uht:UHTProcessingEvent ;
    rdfs:range xsd:decimal ;
    rdfs:label "Holding Time (seconds)" .

uht:qualityTest a rdf:Property ;
    rdfs:domain epcis:ObjectEvent ;
    rdfs:range uht:QualityTestResult .

uht:QualityTestResult a rdfs:Class ;
    rdfs:label "Quality Test Result" .

uht:testType a rdf:Property ;
    rdfs:domain uht:QualityTestResult ;
    rdfs:label "Test Type (acidity, bacteria, protein, fat)" .

uht:testPassed a rdf:Property ;
    rdfs:domain uht:QualityTestResult ;
    rdfs:range xsd:boolean ;
    rdfs:label "Test Passed" .
```

---

## 5. Benefits of GS1 EPCIS Integration

### Interoperability
- **Standard compliance:** Events compatible with any EPCIS 2.0 system
- **Cross-chain compatibility:** Data can be exchanged with other blockchain systems
- **Industry acceptance:** GS1 standards used by 2M+ companies globally

### Rich Semantics
- **What:** Product identifiers (GTIN, EPC, batch/lot)
- **Where:** Locations (SGLN), read points
- **When:** Timestamps with timezone
- **Why:** Business steps, dispositions
- **How:** Sensor data, certifications

### Validation
- **SHACL shapes** ensure data quality
- **Standard code lists** prevent invalid values
- **Machine-readable rules** enable automated validation

---

## 6. Implementation Guide

### Loading Ontologies

```rust
use provchain_org::semantic::rdf_store::RdfStore;

let store = RdfStore::new()?;
store.load_ontology("docs/ontologies/gs1_epcis/epcis.ttl")?;
store.load_ontology("docs/ontologies/gs1_epcis/cbv.ttl")?;
store.load_ontology("docs/ontologies/gs1_epcis/gs1-web-vocab.ttl")?;
```

### Creating EPCIS Events

```rust
let uht_event = r#"
@prefix epcis: <https://ref.gs1.org/epcis/> .
@prefix cbv: <https://ref.gs1.org/cbv/> .

<event/uht-001> a epcis:TransformationEvent ;
    epcis:eventTime "2024-01-15T08:30:00Z"^^xsd:dateTimeStamp ;
    epcis:bizStep cbv:BizStep-creating_class_instance ;
    epcis:inputQuantityList [ 
        epcis:quantity "10000"^^xsd:decimal 
    ] .
"#;

blockchain.add_block(uht_event.as_bytes().to_vec())?;
```

### Validating with SHACL

```rust
use provchain_org::semantic::shacl_validator::ShaclValidator;

let validator = ShaclValidator::new();
validator.load_shapes("docs/ontologies/gs1_epcis/epcis-shacl.ttl")?;
let result = validator.validate(&event_data)?;
```

---

## 7. References

### Official GS1 Resources
- **EPCIS Standard:** https://ref.gs1.org/standards/epcis
- **CBV Standard:** https://ref.gs1.org/standards/cbv
- **GS1 Web Vocabulary:** https://www.gs1.org/voc
- **GitHub Repository:** https://github.com/gs1/EPCIS
- **WebVoc Repository:** https://github.com/gs1/WebVoc

### Standards
- EPCIS 2.0: ISO/IEC 19987:2024
- CBV 2.0: ISO/IEC 19988:2024

---

## 8. Next Steps

1. **Integrate ontologies** into ProvChain UHT demo
2. **Create UHT-specific extensions** for dairy-specific attributes
3. **Implement SHACL validation** for data quality
4. **Add SPARQL queries** for supply chain traceability
5. **Export to EPCIS XML/JSON** for interoperability testing

---

*Report generated for ProvChain GS1 EPCIS UHT Demo enhancement*
