"""Ontology-package-specific emitters for normalized records."""

from __future__ import annotations

from typing import Callable


def escape(value: str | None) -> str:
    return (value or "").replace("\\", "\\\\").replace('"', '\\"')


def infer_milk_type(name: str | None) -> str:
    text = (name or "").lower()
    if "skim" in text:
        return "Skimmed"
    if "semi" in text:
        return "Semi-Skimmed"
    if "organic" in text:
        return "Organic"
    return "Whole"


def derive_security_level(status: str | None) -> str:
    text = (status or "").lower()
    if "shortage" in text:
        return "High"
    return "Standard"


def emit_uht_product(record: dict) -> str:
    attrs = record["attributes"]
    core = record["core"]
    entity_suffix = record["record_id"].replace("food:fdc:", "")
    event_id = f"uhtProduct-{entity_suffix}"
    name = attrs.get("name")
    milk_type = infer_milk_type(name)
    participant = core.get("agent_id") or "agent:brand:unknown-food-owner"

    return f"""@prefix ex: <http://example.org/> .
@prefix uht: <http://provchain.org/uht#> .
@prefix core: <http://provchain.org/core#> .
@prefix trace: <http://provchain.org/trace#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:{event_id} a uht:UHTProduct , core:Product ;
    trace:name "{escape(name)}" ;
    trace:participant "{escape(participant)}" ;
    trace:status "Released" ;
    uht:milkType "{milk_type}" ;
    uht:fatContent "3.5"^^xsd:decimal ;
    uht:proteinContent "3.2"^^xsd:decimal ;
    uht:expiryDate "2026-12-31"^^xsd:date ;
    uht:packageSize "1.0"^^xsd:decimal .
"""


def emit_uht_epcis(record: dict) -> str:
    attrs = record["attributes"]
    core = record["core"]
    entity_suffix = record["record_id"].replace("food:fdc:", "")
    event_id = f"uhtEpcisProduct-{entity_suffix}"
    name = attrs.get("name")
    milk_type = infer_milk_type(name)
    participant = core.get("agent_id") or "agent:brand:unknown-food-owner"
    timestamp = core.get("published_at") or "2026-03-10T00:00:00Z"

    return f"""@prefix ex: <http://example.org/> .
@prefix uht: <http://provchain.org/uht#> .
@prefix core: <http://provchain.org/core#> .
@prefix trace: <http://provchain.org/trace#> .
@prefix epcis: <https://ns.gs1.org/epcis/> .
@prefix cbv: <https://ref.gs1.org/cbv/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:{event_id} a uht:UHTProduct , core:Product , epcis:ObjectEvent ;
    trace:name "{escape(name)}" ;
    trace:participant "{escape(participant)}" ;
    trace:status "Released" ;
    uht:milkType "{milk_type}" ;
    uht:fatContent "3.5"^^xsd:decimal ;
    uht:proteinContent "3.2"^^xsd:decimal ;
    uht:expiryDate "2026-12-31"^^xsd:date ;
    uht:packageSize "1.0"^^xsd:decimal ;
    epcis:eventTime "{escape(timestamp)}"^^xsd:dateTime ;
    epcis:action "ADD" ;
    epcis:bizStep cbv:BizStep-commissioning ;
    epcis:disposition cbv:Disp-active ;
    epcis:readPoint <urn:epc:id:sgln:1234567.12345.1> ;
    epcis:bizLocation <urn:epc:id:sgln:1234567.12345.0> .
"""


def emit_healthcare_device(record: dict, default_location: str = "Registry Catalog") -> str:
    core = record["core"]
    attrs = record["attributes"]
    primary_id = record["identifiers"]["primary"].replace("udi-di:", "")
    event_id = record["record_id"].replace("device:udi-di:", "deviceAudit-")
    timestamp = core.get("published_at") or "2026-03-10T00:00:00Z"
    serial = f"MD-{primary_id[-8:]}" if len(primary_id) >= 8 else f"MD-{primary_id}"

    return f"""@prefix ex: <http://example.org/> .
@prefix healthcare: <http://provchain.org/healthcare#> .
@prefix trace: <http://provchain.org/trace#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:{event_id} a healthcare:MedicalDevice ;
    healthcare:deviceSerialNumber "{escape(serial)}" ;
    trace:name "{escape(attrs.get('name'))}" ;
    trace:status "Sterile" ;
    trace:location "{escape(default_location)}" ;
    trace:timestamp "{escape(timestamp)}"^^xsd:dateTime .
"""


def emit_pharma_storage(record: dict) -> str:
    attrs = record["attributes"]
    event_id = record["record_id"].replace("incident:drug-shortage:", "storageCheck-")
    timestamp = record["core"].get("published_at") or "2026-03-10T00:00:00Z"
    security = derive_security_level(attrs.get("status"))

    return f"""@prefix ex: <http://example.org/> .
@prefix pharma: <http://provchain.org/pharma#> .
@prefix trace: <http://provchain.org/trace#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:{event_id} a pharma:PharmaceuticalStorage ;
    trace:temperature "5.2"^^xsd:decimal ;
    trace:humidity "55.0"^^xsd:decimal ;
    pharma:lightProtection "true"^^xsd:boolean ;
    pharma:controlledSubstance "false"^^xsd:boolean ;
    pharma:securityLevel "{escape(security)}" ;
    trace:recordedAt "{escape(timestamp)}"^^xsd:dateTime .
"""


EMITTERS: dict[str, Callable[..., str]] = {
    "uht": emit_uht_product,
    "uht_epcis": emit_uht_epcis,
    "healthcare_device": emit_healthcare_device,
    "pharma_storage": emit_pharma_storage,
}
