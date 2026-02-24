# Reviewer Report: Computers and Electronics in Agriculture

**Manuscript ID:** [To be assigned]  
**Title:** ProvChain: A GS1 EPCIS-Compliant Blockchain Framework for Ultra-High Temperature (UHT) Milk Supply Chain Traceability with Semantic Web Integration  
**Authors:** [Anonymous for review]  
**Date:** February 24, 2026  
**Reviewer:** Reviewer #1 (Expert in Blockchain & Food Traceability)

---

## Overall Recommendation

**DECISION: Major Revision**

This manuscript presents an interesting integration of GS1 EPCIS standards with blockchain technology for dairy supply chain traceability. The topic is highly relevant to the journal's scope, and the work demonstrates technical sophistication. However, there are significant concerns regarding experimental validation, comparison with existing systems, and clarity of presentation that must be addressed before publication.

---

## Detailed Comments

### 1. Title (Minor Issue)

**Current:** "ProvChain: A GS1 EPCIS-Compliant Blockchain Framework for Ultra-High Temperature (UHT) Milk Supply Chain Traceability with Semantic Web Integration"

**Comment:** The title is accurate but somewhat lengthy. Consider shortening while maintaining clarity. Perhaps:
- "ProvChain: An EPCIS-Compliant Blockchain Framework with Semantic Web Technologies for UHT Milk Traceability"

**Priority:** Low

---

### 2. Abstract (Major Issue - Clarity)

**Strengths:**
- Comprehensive coverage of contributions
- Clear motivation statement
- Specific technical details (65ms latency)

**Weaknesses:**

**Line 84:** The abstract claims "production-ready reference implementation" but the paper later states limitations including single-node RDF store and PoA consensus only. These are not "production-ready" characteristics for enterprise deployment.

**Line 84:** The phrase "addresses critical gaps" is strong but needs more specific evidence in the results section.

**Suggestion:** Remove "production-ready" or qualify it as "prototype demonstration" or "proof-of-concept implementation."

---

### 3. Introduction - Research Gaps (Major Issue)

**Lines 130-138:** The research gaps identified are reasonable, but the justification is weak:

1. **Semantic Gap:** Citation needed for the claim that "most blockchain systems store data as simple key-value pairs."

2. **Standardization Gap:** The paper claims "few blockchain implementations fully comply with GS1 EPCIS" but Table 1 (comparison) only compares with 3 systems. A more comprehensive literature review is needed.

3. **Domain-Specific Gap:** This is the strongest justification. The UHT-specific ontology is indeed novel.

4. **Persistence Gap:** The distinction between "academic prototypes" and the WAL implementation needs quantitative evidence.

**Recommendation:** Strengthen the literature review to provide statistical evidence for these gaps (e.g., "A review of 50 recent papers found only 12% using standardized formats...").

---

### 4. Related Work - Comparison Table (Major Issue)

**Table 1 (Comparison of Systems):**

**Critical Problem:** The comparison criteria are poorly defined:

- What does "Open" column mean? Open source? Open standards? The checkmarks are ambiguous.
- "Persist" column values are inconsistent ("Commercial", "Basic", "WAL"). These are not comparable categories.
- No quantitative metrics for performance comparison.

**Missing Comparisons:**
- HyperLedger Fabric-based systems (most common in industry)
- Ethereum-based traceability solutions
- Recent 2023-2024 publications on food blockchain

**Recommendation:** 
1. Redefine comparison criteria with clear yes/no/metric values
2. Add performance metrics columns (throughput, latency, scalability)
3. Include at least 5-6 more recent systems for comprehensive comparison

---

### 5. System Architecture (Major Issue - Missing Details)

**Section 3 - Architecture Diagram:**

**Line [Architecture figure]:** The TikZ diagram shows layers but lacks:
- Data flow arrows between components
- External system interfaces (ERP, WMS, IoT devices)
- Network topology (peer-to-peer vs client-server)

**Section 3.2 - Ontology Architecture:**

**Critical Missing Information:**
1. Size of ontologies (number of classes, properties, individuals)
2. Reasoning complexity (classification time, consistency checking time)
3. SHACL validation performance (how long for 1000 events?)

**Ontology Reuse:**
The paper mentions FOODON and SCRO but doesn't clarify if these were reused or extended. This is important for semantic interoperability claims.

**Recommendation:**
1. Provide ontology metrics table
2. Clarify relationship with existing ontologies
3. Show reasoning performance benchmarks

---

### 6. Implementation (Major Issue - Lack of Code Availability)

**Section 4.1 - Technology Stack:**

**Critical Issue:** The paper states "Open-source reference implementation" but:
- No GitHub repository URL provided
- No code availability statement in Data Availability section
- The README says "[To be filled by authors]"

**Lines [Code listings]:** The code snippets in Listings 1-3 are illustrative but:
- No line numbers referenced in text
- No explanation of error handling
- No testing methodology shown

**Section 4.3 - UHT Supply Chain Demo:**

**Missing Critical Details:**
1. How were the 8 phases validated? Real data or synthetic?
2. What IoT sensors were used (if any)?
3. How was the cold chain temperature data captured?

**Recommendation:**
1. Provide actual repository URL
2. Include unit test coverage metrics
3. Describe demo validation methodology

---

### 7. Evaluation (Major Issue - Weak Experimental Design)

**Section 5 - Performance Metrics:**

**Table 3 (Performance Results):**

**Critical Problems:**

1. **Block creation time (6.74s for 113 blocks):** This seems extremely slow. Bitcoin creates blocks every 10 minutes, Ethereum every 12 seconds. 6.74s for 113 blocks = 0.06s/block is fast, but the comparison baseline is missing.

2. **Events per second (16.76 events/s):** This is very low for a blockchain system. Hyperledger Fabric can handle 3000+ TPS. The bottleneck needs explanation.

3. **Average event latency (65ms):** This is reasonable but compared to what? No baseline comparison provided.

4. **No statistical analysis:** Where are the standard deviations? Min/max values? Confidence intervals?

**Section 5.2 - Standards Compliance:**

**Line [Compliance]:** The checkmarks in Table 4 are self-assessed. Independent validation would strengthen the paper.

**Section 5.3 - Semantic Validation:**

**Major Concern:** Only 100 test events? For a production system, this is insufficient. Need:
- Scalability testing (10,000+ events)
- Stress testing (concurrent event submission)
- Long-running test (24+ hours continuous operation)

**Recommendation:**
1. Expand test scale to at least 10,000 events
2. Provide statistical analysis with standard deviations
3. Include comparison with baseline systems
4. Add stress testing results

---

### 8. Discussion - Limitations (Minor Issue - Honesty Appreciated)

**Section 6.2 - Limitations:**

The authors honestly identify limitations, which is commendable. However:

1. **Single-node RDF store:** This contradicts the "production-ready" claim in the abstract.

2. **PoA only:** No discussion of why PoW or PoS weren't considered or implemented.

3. **Adoption barriers:** Generic discussion. Need specific cost estimates or deployment complexity analysis.

**Recommendation:** Quantify the limitations (e.g., "Current implementation handles 100 events/second; for 1000 events/second, distributed RDF would be required, estimated 3x development effort").

---

### 9. References (Minor Issue)

**Total:** 22 references - This is low for a comprehensive survey paper. 

**Missing Key References:**

1. **Recent EPCIS 2.0 papers (2023-2024):** The most recent EPCIS citation is 2021. EPCIS 2.0 was ratified in 2022; there should be recent papers.

2. **SHACL validation in supply chains:** Only one SHACL paper cited (Garcia 2022). More recent work exists.

3. **Rust-based blockchain systems:** Since the implementation is in Rust, comparisons with other Rust blockchain projects (e.g., Substrate) would be relevant.

4. **Food traceability standards beyond GS1:** ISO 22005, GLOBALG.A.P. - how does the system align with these?

**Recommendation:** Expand to at least 35-40 references with more recent (2023-2024) publications.

---

### 10. Writing and Presentation (Minor Issues)

**Overall Quality:** Generally well-written with clear technical descriptions.

**Specific Issues:**

1. **Line 65 (Title):** "Ultra-High Temperature (UHT)" - acronym defined twice (also in keywords).

2. **Line 84 (Abstract):** "65ms average" - should be "average of 65 ms" for clarity.

3. **Section 3.2:** Use of \texttt in LaTeX for technical terms is good, but inconsistent in some sections.

4. **Figure 1:** The architecture diagram is clear but needs a legend explaining arrow types.

5. **Tables:** All tables need consistent formatting (some use \toprule, others don't).

---

## Major Concerns Summary

### 1. **Experimental Validation Weakness (CRITICAL)**
- Only 100 test events
- No statistical analysis
- No comparison with established baselines
- "Production-ready" claim not supported by evidence

### 2. **Comparison Inadequacy (CRITICAL)**
- Table 1 compares only 4 systems
- Missing key competitors (Hyperledger Fabric, Ethereum-based)
- No quantitative performance comparison

### 3. **Code Availability (CRITICAL)**
- Claims open-source but no repository provided
- Data Availability section is empty

### 4. **Scalability Concerns (MAJOR)**
- Single-node RDF store
- 16.76 events/second is very low
- No discussion of horizontal scaling

---

## Positive Aspects

1. **Novel Integration:** The combination of EPCIS + Blockchain + Semantic Web is genuinely innovative.

2. **UHT-Specific Ontology:** Domain-specific contribution is valuable and well-motivated.

3. **WAL Persistence:** Technical sophistication in persistence layer.

4. **Standards Compliance:** Attention to GS1 standards is important for practical adoption.

5. **Clear Structure:** Well-organized manuscript with logical flow.

---

## Specific Questions for Authors

1. **Q1:** Can you provide the actual GitHub repository URL for the open-source implementation?

2. **Q2:** How does your 16.76 events/second throughput compare to Hyperledger Fabric (3000+ TPS) and what causes this 170x difference?

3. **Q3:** Was the UHT demo validated with real dairy industry data or synthetic data? If synthetic, how realistic are the parameters?

4. **Q4:** Why was Proof-of-Authority chosen over Practical Byzantine Fault Tolerance (PBFT) or other consensus mechanisms suitable for permissioned networks?

5. **Q5:** How does your UHT ontology relate to existing food ontologies (FOODON, FAO)? Did you reuse or extend their classes?

6. **Q6:** What is the reasoning performance for complex OWL2 constructs (property chains with 5+ links)?

7. **Q7:** Can you provide a cost analysis comparing deployment of ProvChain vs. IBM Food Trust for a mid-size dairy processor?

---

## Action Items for Authors

### Required for Major Revision:

- [ ] **R1:** Expand experimental evaluation to at least 10,000 events with statistical analysis
- [ ] **R2:** Provide comprehensive comparison with at least 8-10 competing systems
- [ ] **R3:** Clarify "production-ready" claims or replace with "proof-of-concept"
- [ ] **R4:** Provide open-source repository URL with documentation
- [ ] **R5:** Expand literature review to 35+ references including 2023-2024 papers
- [ ] **R6:** Add scalability analysis and bottleneck identification
- [ ] **R7:** Clarify relationship with existing ontologies (FOODON, SCRO)

### Recommended for Minor Revision:

- [ ] **S1:** Add system architecture diagram with data flows
- [ ] **S2:** Include ontology metrics (classes, properties, reasoning time)
- [ ] **S3:** Provide cost/complexity analysis for deployment
- [ ] **S4:** Add graphical abstract
- [ ] **S5:** Proofread for consistent formatting

---

## Final Assessment

**Novelty:** High (integration of EPCIS + Blockchain + Semantic Web)  
**Technical Quality:** Medium (good architecture, weak validation)  
**Presentation:** Good (clear structure, readable prose)  
**Significance:** Medium-High (relevant to food safety challenges)

**Recommendation: MAJOR REVISION**

The paper presents interesting technical work but requires substantial strengthening of experimental validation and comparison with existing systems before it can be considered for publication. The "production-ready" claims are not supported by the current evidence. If the authors can address the critical concerns (R1-R7), this could become a valuable contribution to the journal.

---

**Reviewer Signature:** Reviewer #1  
**Expertise:** Blockchain systems, Food supply chain traceability, Semantic web technologies  
**Conflict of Interest:** None declared
