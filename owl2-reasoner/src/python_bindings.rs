//! Python Bindings for OWL2 Reasoner with EPCIS Integration
//!
//! This module provides Python bindings using PyO3 to expose the OWL2 reasoner
//! and EPCIS functionality to Python applications.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use pyo3::types::PyString;
use pyo3::wrap_pyfunction;
use std::collections::HashMap;
use std::sync::Arc;

use crate::epcis::*;
use crate::epcis_parser::*;
use crate::reasoning::SimpleReasoner;
use crate::Ontology;

/// Python wrapper for EPCIS events
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyEPCISEvent {
    #[pyo3(get)]
    pub event_id: String,
    #[pyo3(get)]
    pub event_type: String,
    #[pyo3(get)]
    pub event_time: String,
    #[pyo3(get)]
    pub epcs: Vec<String>,
    #[pyo3(get)]
    pub biz_step: Option<String>,
    #[pyo3(get)]
    pub disposition: Option<String>,
    #[pyo3(get)]
    pub action: String,
}

#[pymethods]
impl PyEPCISEvent {
    #[new]
    pub fn new(
        event_id: String,
        event_type: String,
        event_time: String,
        epcs: Vec<String>,
        biz_step: Option<String>,
        disposition: Option<String>,
        action: String,
    ) -> Self {
        Self {
            event_id,
            event_type,
            event_time,
            epcs,
            biz_step,
            disposition,
            action,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "PyEPCISEvent(event_id='{}', event_type='{}', epcs=[...], action='{}')",
            self.event_id, self.event_type, self.action
        )
    }

    pub fn to_dict(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        dict.set_item("event_id", &self.event_id)?;
        dict.set_item("event_type", &self.event_type)?;
        dict.set_item("event_time", &self.event_time)?;
        dict.set_item("epcs", &self.epcs)?;
        dict.set_item("biz_step", &self.biz_step)?;
        dict.set_item("disposition", &self.disposition)?;
        dict.set_item("action", &self.action)?;
        Ok(dict.into())
    }
}

/// Python wrapper for EPCIS parser
#[pyclass]
pub struct PyEPCISParser {
    parser: EPCISDocumentParser,
}

#[pymethods]
impl PyEPCISParser {
    #[new]
    pub fn new() -> Self {
        Self {
            parser: EPCISDocumentParser::default(),
        }
    }

    pub fn parse_xml_file(&self, file_path: &str) -> PyResult<Vec<PyEPCISEvent>> {
        match self.parser.parse_xml_file(file_path) {
            Ok(events) => Ok(events
                .into_iter()
                .map(|e| PyEPCISEvent {
                    event_id: e.event_id,
                    event_type: e.event_type,
                    event_time: e.event_time,
                    epcs: e.epcs,
                    biz_step: e.biz_step,
                    disposition: e.disposition,
                    action: e.action,
                })
                .collect()),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to parse EPCIS file: {}",
                e
            ))),
        }
    }

    pub fn parse_xml_string(&self, xml_content: &str) -> PyResult<Vec<PyEPCISEvent>> {
        match self.parser.parse_xml_str(xml_content) {
            Ok(events) => Ok(events
                .into_iter()
                .map(|e| PyEPCISEvent {
                    event_id: e.event_id,
                    event_type: e.event_type,
                    event_time: e.event_time,
                    epcs: e.epcs,
                    biz_step: e.biz_step,
                    disposition: e.disposition,
                    action: e.action,
                })
                .collect()),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to parse EPCIS XML: {}",
                e
            ))),
        }
    }

    pub fn extract_all_epcs(&self, events: Vec<PyEPCISEvent>) -> Vec<String> {
        let simple_events: Vec<EPCISSimpleEvent> = events
            .into_iter()
            .map(|e| EPCISSimpleEvent {
                event_id: e.event_id,
                event_type: e.event_type,
                event_time: e.event_time,
                epcs: e.epcs,
                biz_step: e.biz_step,
                disposition: e.disposition,
                action: e.action,
            })
            .collect();
        self.parser.extract_all_epcs(&simple_events)
    }

    pub fn extract_events_by_type(&self, events: Vec<PyEPCISEvent>) -> HashMap<String, usize> {
        let simple_events: Vec<EPCISSimpleEvent> = events
            .into_iter()
            .map(|e| EPCISSimpleEvent {
                event_id: e.event_id,
                event_type: e.event_type,
                event_time: e.event_time,
                epcs: e.epcs,
                biz_step: e.biz_step,
                disposition: e.disposition,
                action: e.action,
            })
            .collect();
        self.parser.extract_events_by_type(&simple_events)
    }
}

/// Python wrapper for OWL2 reasoner
#[pyclass]
pub struct PyOWL2Reasoner {
    reasoner: SimpleReasoner,
}

#[pymethods]
impl PyOWL2Reasoner {
    #[new]
    pub fn new() -> Self {
        let ontology = Ontology::new();
        Self {
            reasoner: SimpleReasoner::new(ontology),
        }
    }

    pub fn load_epcis_events(&mut self, events: Vec<PyEPCISEvent>) -> PyResult<()> {
        let simple_events: Vec<EPCISSimpleEvent> = events
            .into_iter()
            .map(|e| EPCISSimpleEvent {
                event_id: e.event_id,
                event_type: e.event_type,
                event_time: e.event_time,
                epcs: e.epcs,
                biz_step: e.biz_step,
                disposition: e.disposition,
                action: e.action,
            })
            .collect();

        let parser = EPCISDocumentParser::default();
        match parser.to_ontology(&simple_events) {
            Ok(ontology) => {
                self.reasoner = SimpleReasoner::new(ontology);
                Ok(())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to create ontology from EPCIS events: {}",
                e
            ))),
        }
    }

    pub fn is_consistent(&self) -> PyResult<bool> {
        match self.reasoner.is_consistent() {
            Ok(result) => Ok(result),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Consistency check failed: {}",
                e
            ))),
        }
    }

    pub fn get_statistics(&self) -> PyResult<HashMap<String, usize>> {
        let mut stats = HashMap::new();
        stats.insert("classes", self.reasoner.ontology.classes().len());
        stats.insert("object_properties", self.reasoner.ontology.object_properties().len());
        stats.insert("data_properties", self.reasoner.ontology.data_properties().len());
        stats.insert("individuals", self.reasoner.ontology.named_individuals().len());
        stats.insert("axioms", self.reasoner.ontology.axioms().len());
        Ok(stats)
    }

    pub fn validate_el_profile(&self) -> PyResult<bool> {
        match self.reasoner.validate_profile(crate::profiles::Owl2Profile::EL) {
            Ok(result) => Ok(result.is_valid),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "EL profile validation failed: {}",
                e
            ))),
        }
    }

    pub fn validate_ql_profile(&self) -> PyResult<bool> {
        match self.reasoner.validate_profile(crate::profiles::Owl2Profile::QL) {
            Ok(result) => Ok(result.is_valid),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "QL profile validation failed: {}",
                e
            ))),
        }
    }

    pub fn validate_rl_profile(&self) -> PyResult<bool> {
        match self.reasoner.validate_profile(crate::profiles::Owl2Profile::RL) {
            Ok(result) => Ok(result.is_valid),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RL profile validation failed: {}",
                e
            ))),
        }
    }
}

/// EPCIS data generator for Python
#[pyclass]
pub struct PyEPCISGenerator {
    config: EPCISDataConfig,
}

#[pymethods]
impl PyEPCISGenerator {
    #[new]
    pub fn new(scale: &str) -> PyResult<Self> {
        let config = match scale {
            "small" => EPCISDataConfig::small_scale(),
            "medium" => EPCISDataConfig::medium_scale(),
            "large" => EPCISDataConfig::large_scale(),
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Scale must be 'small', 'medium', or 'large'",
                ))
            }
        };
        Ok(Self { config })
    }

    pub fn generate_events(&self, count: usize) -> Vec<PyEPCISEvent> {
        let mut events = Vec::new();
        for i in 0..count {
            let event = PyEPCISEvent {
                event_id: format!("event_{}", i),
                event_type: "ObjectEvent".to_string(),
                event_time: "2023-01-01T00:00:00Z".to_string(),
                epcs: vec![format!("urn:epc:id:sgtin:0614141.107346.{}", i)],
                biz_step: Some("urn:epcglobal:cbv:bizstep:receiving".to_string()),
                disposition: Some("urn:epcglobal:cbv:disp:in_progress".to_string()),
                action: "ADD".to_string(),
            };
            events.push(event);
        }
        events
    }

    pub fn get_config_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        info.insert("scale".to_string(), format!("{:?}", self.config.scale));
        info.insert(
            "participant_count".to_string(),
            self.config.participant_count.to_string(),
        );
        info.insert(
            "time_span_seconds".to_string(),
            self.config.time_span.as_secs().to_string(),
        );
        info.insert(
            "include_extensions".to_string(),
            self.config.include_extensions.to_string(),
        );
        info
    }
}

/// Python module functions
#[pyfunction]
pub fn parse_epcis_xml_file(file_path: &str) -> PyResult<Vec<PyEPCISEvent>> {
    let parser = PyEPCISParser::new();
    parser.parse_xml_file(file_path)
}

#[pyfunction]
pub fn parse_epcis_xml_string(xml_content: &str) -> PyResult<Vec<PyEPCISEvent>> {
    let parser = PyEPCISParser::new();
    parser.parse_xml_string(xml_content)
}

#[pyfunction]
pub fn create_reasoner() -> PyOWL2Reasoner {
    PyOWL2Reasoner::new()
}

#[pyfunction]
pub fn create_epcis_generator(scale: &str) -> PyResult<PyEPCISGenerator> {
    PyEPCISGenerator::new(scale)
}

/// Python module definition
#[pymodule]
fn owl2_reasoner_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEPCISEvent>()?;
    m.add_class::<PyEPCISParser>()?;
    m.add_class::<PyOWL2Reasoner>()?;
    m.add_class::<PyEPCISGenerator>()?;
    m.add_function(wrap_pyfunction!(parse_epcis_xml_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_epcis_xml_string, m)?)?;
    m.add_function(wrap_pyfunction!(create_reasoner, m)?)?;
    m.add_function(wrap_pyfunction!(create_epcis_generator, m)?)?;
    Ok(())
}