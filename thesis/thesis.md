## THESIS TITLE AND RESEARCH OUTLINE

## 1. Student Name and Surname / Code

(Thai) นายอนุสรณ์ ใจแก้ว

(English) Mr. Anusorn Chaikaew

Student Code 640551018

## 2. Thesis Title

(Thai) การเพิ'มประสิทธิภาพบล็อกเชนด้วยการฝังออนโทโลยีและกราฟความรู้สําหรับการตรวจสอบ ย้อนกลับของข้อมูล

(English) Enhancement of Blockchain with Embedded Ontology and Knowledge Graph for Data Traceability

## 3.  Advisory Committee/Thesis Advisor

Associate Professor Dr. Ekkarat Boonchieng Advisor

Assistant Professor Dr. Sukhuntha Osiriphun Co-advisor

Assistant Professor Dr.

Varin Chouvatut Co-advisor

## 4. Principles, Theory, Rationale, and/or Hypotheses

Blockchains  are  best  known  for  their  critical  role  in  cryptocurrency  systems  like

Bitcoin. A blockchain is a chain of data made up of blocks that, when filled, are connected to the block that came before them. Transactions are irreversible once they are recorded and can

not  be  altered  retroactively.  When  used  in  a  decentralized  manner,  this  data  structure automatically creates an irreversible timeline of data. The world is becoming more and more

interested  in  blockchain  technology.  Blockchain  introduces  novel  concepts  in  various industries, but it is most prominent in the financial sector, where it allows for decentralized

financial  transactions.  Smart  contracts  and  the  most  recent  developments  in  Blockchain technology  are  being  used  to  enable  decentralized  applications  [1].  Emerging  financial

concepts  like  Decentralized  Finance,  GameFi,  NFT  (Non-Fungible  Token),  and  Metaverse technology depend more on blockchain technology and smart contracts [2].

Data traceability ensures your data is traceable throughout the entire landscape. This lets you trace data back to its source. To make precise and accurate insights, track every data transition, dead-end, and link. Source systems and reporting need data traceability. It caters to source-to-target data. Without data traceability, you can't check that all data is correct, affecting

future  decisions.  Traceability  protects  the  integrity  of  your  information  because  you  can observe its journey from origin to the present.

With the feature immutable, decentralized, and distributed and including the chain of a block of Blockchain, it is fit to use Blockchain for infrastructure to support many kinds of business that require the traceability of transactions such as supply chain or medical data in the health care system.

However, blockchain did not answer all questions, such as, If a developer wanted to store other structured data on a blockchain, they would need to first serialize it to binary, just like on Ethereum, which typically serializes data structures into JSON objects and embeds them via  Solidity  scripts  (Sopek  et  al.  2018).  In  addition,  it  made  it  difficult  to  conduct  a  query because the object had to be extracted from the blocks and then restored to the correct type before it could be used. This process was laborious and took a lot of time for applied blockchain to store other transactions that did not relate to finances and have a lot of transactions such as in the supply chain system or medical data. The other major problem is how data stored in different formats interchange between the different chain architectures.

Figure 1. classic blockchain architecture.

<!-- image -->

The W3C is expanding the WWW with the Semantic Web, often called Web 3.0. (not to  be  confused  with  Web3).  It  makes  Internet  data  machine-readable;  "semantic"  means machine-processable. "Web" is a navigable space of interconnected objects having URI-toresource  mappings.  For  the  Semantic  Web  to  work,  computers  need  structured  data  and inference rules for automated reasoning. Linked Open Data (LOD) is graph-based data that can be linked across servers. Tim Berners-Lee outlined the Four Rules of Linked Data in 2006.

Block n+2Transactions

URIs are names. Use HTTP URIs to make names searchable. Provide helpful information about URIs utilizing standards (RDF*, SPARQL). Include URI links so people can learn more. LOD helps  humans  and  machines  access  and  analyze  data  across  servers.  The  Semantic  Web transforms from a linked-document space to a linked-information space. Which enables a vast network of machine-processable meaning. Linked Open Data includes:

- ∞ Factual data about specific entities and concepts
- ∞ Ontologies - semantic schemata defining:
- ∞ Classes of objects (e.g., Person, Organization, Location, and Document);
- ∞ Relationship types (e.g., a parent of or a manufacturer of);
- ∞ Attributes (e.g., the DoB of a person or the population of a geographic region).

An ontology is a set of concepts and how they relate to each other. Ontologies are a way to represent knowledge that can be shared and used again. They can also include new information about a domain.

The ontology data model can be used to make a knowledge graph, a list of things whose types, nodes, and edges show relationships. The ontology allows the knowledge graph to gather data by describing a domain's knowledge structure. Ontologies are essential because they make sure that everyone understands the same information and because they make explicit domain assumptions. Because of this, the interconnectedness and interoperability of the model make it an excellent tool for helping improve blockchain access and query data.

Ontologies also improve the quality of data by making metadata and provenance better. The most important thing about ontologies is that they build relationships between concepts, making it possible for computers to reason about data automatically.

Such reasoning is easy to add to semantic graph databases that use ontologies as their semantic schemata. On top of that, ontologies work like a "brain." They are "work and reason" with ideas and connections similar to how people think about these things. In addition to being able  to  reason,  ontologies  also  make  it  easier  to  move  from  one  concept  to  another  in  the structure of the ontology.

Ontologies also give us a way to represent any data format, whether unstructured, semistructured, or structured. Integrating data, mine concepts, and text more accessible and does data-driven  analytics  inside  the  blockchain  to  make  blockchain  easier  to  retrieve  data  for traceability.

Hence, blockchain embedded with ontology and knowledge graph may enhance the blockchain in data traceability and make it easier to support various data structures to store and retrieve data.

## 5. Literature Review

Ontologies have been applied to supply chains in a wide variety of ways. Most of the existing research focuses on using ontologies to represent the knowledge to support better supply chains, such as representing the information sharing between parties when tracing [7]. Some researchers use ontology to represent the knowledge to help define the risk in the supply chain [8].

<!-- image -->

Year

Figure 2. word growth of blockchain related to ontology

The popularity of blockchain topics in the academic area is rising. Figure 2. shows the word growth of blockchain related to ontology topics with peak in 2019 and Figure 3. show number of research articles in Scopus search with the keywords "blockchain" and "traceability.' The rise started in 2017.

Figure 3. number of research article on Scopus using the keyword blockchain and traceability.

<!-- image -->

There  are  twenty-one documents  found  when  searching  with  keywords  'blockchain'  and 'ontology'  on  the  Scopus  database  which  means  not  much  ontology  to  be  applied  to Blockchain  technology.  However,  some  valuable  papers  are  to  review  and  used  for  this research  topic.  Semantic  Blockchain  project  [3]  BLONDiE  is  an  ontology  applied  to blockchain  data  in  OWL/RDF  format.  Another  Blockchain  with  an  adopted  ontology  is GraphChain [4].  It  supports  RDF  data  format  and  implementations  using  Java,  .NET,  and node.js technologies.  Not only data level, but this research [5] also proposes a method to use ontology  such  as  SWRL  (Semantic  Web  Rule  language)  to  define  rules  to  model  DApps (Decentralized Application).  However, most papers that adopted blockchain ontology do not consider using knowledge graphs.

Ontology and Knowledge graphs can help protect data and resources against breaches or improper modifications. This research [6] proposes a novel framework to use knowledge graphs to develop an automated access-control and audit mechanism that enforces users' data privacy policies while sharing their data with third parties.

Table 1. main feature of existing research relate ontology and blockchain

| Title               | Main focus        | Data Owner Permission Control   | Support Multi Chain   | Configure Consensus   | Support Ontology   |
|---------------------|-------------------|---------------------------------|-----------------------|-----------------------|--------------------|
| Hector &Boris [3]   | General Purpose   | NO                              | NO                    | NO                    | YES                |
| Sopek et al. [4]    | General Purpose   | NO                              | NO                    | NO                    | YES                |
| Besancon et al. [5] | General Purpose   | NO                              | NO                    | NO                    | YES                |
| Joshi &Banerjee [6] | General Purpose   | NO                              | NO                    | NO                    | YES                |
| This research       | Data Traceability | YES                             | YES                   | YES                   | YES                |

Table 1. Show the main feature of existing research that applies ontology to the blockchain compare with this research. Most of all, it focuses on general purpose, which is different from this research, which will try to focus on data traceability with the ability to give the owner control data permission. Moreover, this research attempts to invent data traceability between blockchains and support configuring the consensus protocol.

## 6. Research Objectives

1. To design a blockchain with the new data structure
2. To develop a blockchain with embedded ontology for data traceability
3. To test the usability of data traceability on the blockchain with an ontology

## 7. Usefulness of the Research (Theoretical and/or Applied)

This research is suitable for businesses requiring data traceability, such as supply chain and hospital medical data.

## With Main contribution are

- -Ontology and knowledge graph with Blockchain
- -Multi Consensus with configuration mechanism
- -Permission control in Blockchain data sharing
- -Cross Chain data interchange

## 8. Methodology, Scope, and Research Plan

This research focuses on building a new blockchain that supports data traceability on a high-speed query and is easier to retrieve than traditional blockchains; and also controls the permission  of  data  for  sharing  in  public,  which  is  the  pain  point  of  current  blockchain technology. Moreover, this new blockchain can support multiple existing consensus protocols by configuration. However, this research has the tradeoff of not supporting smart contracts for programming  on  the  blockchain.  The  main  essential  questions  in  the  supply  chain  for traceability and the need for a solution are the following:

- -How can we trust data from another party in a network?
- -How to control the shared data on different levels (visibility control)
- -How to query the transaction more conveniently than traditional blockchain
- -How to transfer data between other clusters of supply chains that are not on the same network.

This research should solve the questions stated above.

## Methodology

The overview of the methodology proposed to achieve the objective of this research is shown in Figure 4.

Figure 4. overview of component of the new Blockchain with embed ontology for support traceability in supply chain

<!-- image -->

In this research, we also split the principal methodology into three main phases.

Phase I is designing a blockchain with the new data structure.

Figure. 5 layer of storing  transaction on blockchain

<!-- image -->

Figure. 6  layer of   retrieving data from the blockchain.

<!-- image -->

In  this  phase,  we  study  various  existing  blockchain  architectures,  semantic  web technology, a knowledge graph, and other related technology to identify the weakness and fulfill  our  blockchain  architecture.  The  core  idea  of  this  research  is  to  embed  the  ontology inside  the  data  block  as  Figure  4.  We  will  compare  with  existing  the  blockchain  such  as Ethereum, and Hyper ledger Fabric to test the performance when doing the data traceability with the new architect as Figure 5.

## Phase II Developing a Blockchain with embed ontology for data traceability

In this phase, we develop the Blockchain following by design in the previous step by using various tools and frameworks as following

Table 2. tools and frameworks in this research

| Programming Language   | Infrastructure      | Frameworks         | Other                                              |
|------------------------|---------------------|--------------------|----------------------------------------------------|
| Java Clojure Go Rust   | Docker, Kubernetes, | JavaScript/Node.js | Neo4J, Graph DB, Apache Jena, RDF4J, Protégé, Hozo |

## Phase III Testing the usability of data traceability on the blockchain with an ontology

This phase will split into two sub-testing phases as follows

1. Use STLC 6 phase of testing

<!-- image -->

Requirement Analysis - When finished design, the system starts a high-level analysis concerning the AUT (Application under Test).

Test Planning - plans the strategy and approach.

Test Case Designing - Develop the test cases based on the scope and criteria of research.

Test Environment Setup - When the integrated environment is ready to validate the product.

Test Execution - Real-time validation of product and finding bugs.

Test Closure - Once testing is completed, the matrix, reports, and results are documented.

## 2. Deploy and test with actual data

- a. Supply chain
- b. Medical data

## Scope of this research

In this research we would like to redesign the distributed ledgers system based on the blockchain concept which includes the following scope/feature

1. This system focuses on data traceability, especially in the food supply chain system.
2. It has permission control by the owner of data for sharing data in private/public.
3. It will support data interchange between two systems.
4. It stores data on ontology (RDF) structure with a knowledge graph for data traceability capability.

## Performance Evaluation

This research decide to compare the performance with the following topics

1. Compare read/write speed (including hash digest) with the traditional blockchain
2. -Hyperledger Fabric (PBFT)
3. -Ethereum (PoW, PoS)
2. Compare read/write speed with existing graph database
5. -Neo4J / TigerGraph/ FlureeDB
3. Compare the sharing data algorithm with existing algorithm on Blockchain

## Research Plan

| Activities                                        | st Year   | st Year   | st Year   | st Year   | nd Year   | nd Year   | nd Year   | nd Year   | 3 rd Year   | 3 rd Year   | 3 rd Year   | 3 rd Year   |
|---------------------------------------------------|-----------|-----------|-----------|-----------|-----------|-----------|-----------|-----------|-------------|-------------|-------------|-------------|
| Activities                                        | 1-3       | 4-6       | 7-9       | 10-12     | 1-3       | 4-6       | 7-9       | 10-12     | 1-3         | 4-6         | 7-9         | 10-12       |
| Present Survey research at International Conf     |           |           |           |           |           |           |           |           |             |             |             |             |
| Research Proposal                                 |           |           |           |           |           |           |           |           |             |             |             |             |
| Design Blockchain                                 |           |           |           |           |           |           |           |           |             |             |             |             |
| Submit a 1 st research paper                      |           |           |           |           |           |           |           |           |             |             |             |             |
| Implement Blockchain                              |           |           |           |           |           |           |           |           |             |             |             |             |
| Submit a 2 nd research paper                      |           |           |           |           |           |           |           |           |             |             |             |             |
| Test Blockchain system on actual transaction data |           |           |           |           |           |           |           |           |             |             |             |             |

## 9. Research Location

Department of Computer Science Faculty of Science,  Chiang Mai University

## 10. Research Duration

3 Years

## 11. References

- [1] Anusorn, C. Sukhuntha, O, Boonchieng, E.: Blockchain for supply chain traceability: a survey: 2022 12th IIAI International Congress on Advanced Applied Informatics. IEEE, Kanazawa, Japan.
- [2] Busayatananphon, C., Boonchieng, E.: Financial Technology DeFi Protocol: A Review. In: 2022 Joint International Conference on Digital Arts, Media and Technology with ECTI Northern Section Conference on Electrical, Electronics, Computer and Telecommunications Engineering (ECTI DAMT &amp; NCON). pp. 267272. IEEE, Chiang Rai, Thailand (2022)
- [3] Hector, U.-R., &amp; Boris, C.-L. (2020). BLONDiE: Blockchain Ontology with Dynamic Extensibility (arXiv:2008.09518). arXiv. http://arxiv.org/abs/2008.09518
- [4] M. Sopek, P. Gradzki, W. Kosowski, D. Kuziski, R. Trójczak, and R. Trypuz, 'GraphChain: A Distributed Database with Explicit Semantics and Chained RDF Graphs,' in Companion of the The Web Conference 2018 on The Web Conference 2018 - WWW '18, Lyon, France, 2018, pp. 1171-1178. doi: 10.1145/3184558.3191554.
- [5] L. Besancon, C. F. Da Silva, P. Ghodous, and J.-P. Gelas, 'A Blockchain Ontology for DApps Development,' IEEE Access, vol. 10, pp. 49905-49933, 2022, doi: 10.1109/ACCESS.2022.3173313.
- [6] K. P. Joshi and A. Banerjee, 'Automating Privacy Compliance Using Policy Integrated Blockchain,' Cryptography, vol. 3, no. 1, p. 7, Feb. 2019, doi: 10.3390/cryptography3010007.
- [7] Salampasis, Michail, Dimitrios Tektonidis, and Eleni P. Kalogianni. 2012. 'TraceALL: A Semantic Web Framework for Food Traceability Systems' edited by A. Matopoulos. Journal of Systems and Information Technology 14(4):302-17. doi: 10.1108/13287261211279053.
- [8] Cao, Shoufeng, Kim Bryceson, and Damian Hine. 2019. 'An Ontology-Based Bayesian Network Modelling for Supply Chain Risk Propagation.' Industrial Management &amp; Data Systems 119(8):1691-1711. doi: 10.1108/IMDS-01-2019-0032.

## 12.  Thesis Advisors

The undersigned have read and approved this thesis proposal and have agreed to act as the Thesis Advisory Committee in the respective capacities mentioned below.

<!-- image -->

(Signed)…………………………………………………   Chairperson ( Associate Professor Dr. Ekkarat Boonchieng )

(Signed) ………………………………………………… Co-advisor ( Assistant Professor Dr. Sukhuntha Osiriphun )

(Signed)…………………………………………………   Co-advisor

( Assistant Professor Dr. Varin Chouvatut )