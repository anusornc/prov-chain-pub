#!/usr/bin/env python3
"""
Baseline Benchmark Runner for Journal Publication
Compares ProvChainOrg against Neo4j, Jena, and Ethereum

Author: Mr. Anusorn Chaikaew (Student Code: 640551018)
Thesis: "Enhancement of Blockchain with Embedded Ontology and Knowledge Graph for Data Traceability"
Date: 2026-01-18

Academic Integrity: All benchmarks use REAL experimental data
"""

import time
import statistics
import requests
import json
import csv
import os
import jwt
from typing import List, Dict, Tuple, Optional
from neo4j import GraphDatabase
from web3 import Web3
import pandas as pd

# =============================================================================
# CONFIGURATION
# =============================================================================

NEO4J_URI = os.getenv("NEO4J_URI", "bolt://localhost:7687")
NEO4J_USER = os.getenv("NEO4J_USER", "neo4j")
NEO4J_PASSWORD = os.getenv("NEO4J_PASSWORD", "benchmark_password")

JENA_SPARQL = os.getenv("JENA_SPARQL", "http://localhost:3030/ds/query")
JENA_UPDATE = os.getenv("JENA_UPDATE", "http://localhost:3030/ds/update")

ETH_RPC = os.getenv("ETH_RPC", "http://localhost:8545")

PROVCHAIN_URL = os.getenv("PROVCHAIN_URL", "http://localhost:8080")
PROVCHAIN_JWT_SECRET = os.getenv(
    "JWT_SECRET",
    "development-secret-key-min-32-chars-for-demo-mode-only"
)

DATASET_PATH = os.getenv("DATASET_PATH", "./datasets")
RESULTS_PATH = os.getenv("RESULTS_PATH", "./results")
ITERATIONS = int(os.getenv("ITERATIONS", "100"))

# =============================================================================
# BENCHMARK DATASETS (Supply Chain Traceability)
# =============================================================================

BENCHMARK_TRIPLES = [
    ("100", 100),
    ("500", 500),
    ("1000", 1000),
    ("5000", 5000),
]

# Sample SPARQL queries with proper prefix declarations
SPARQL_QUERIES = {
    "simple_select": """
        PREFIX : <http://example.org/>
        SELECT ?s ?p ?o
        WHERE {
            ?s ?p ?o .
            ?s a :Product .
        }
        LIMIT 10
    """,
    "type_query": """
        PREFIX : <http://example.org/>
        SELECT ?s (COUNT(?o) AS ?count)
        WHERE {
            ?s a :Product .
            ?s ?p ?o .
        }
        GROUP BY ?s
        LIMIT 10
    """,
    "join_query": """
        PREFIX : <http://example.org/>
        SELECT ?t1 ?t2
        WHERE {
            ?t1 :involves ?p .
            ?t2 :involves ?p .
            FILTER (?t1 != ?t2)
        }
        LIMIT 10
    """,
    "complex_join": """
        PREFIX : <http://example.org/>
        SELECT ?s1 ?s2 ?p1 ?p2
        WHERE {
            ?s1 :involves ?p1 .
            ?s2 :involves ?p2 .
            ?p1 :suppliedBy ?supplier .
            ?p2 :suppliedBy ?supplier .
            FILTER (?s1 != ?s2)
        }
        LIMIT 10
    """
}

# =============================================================================
# BASELINE 1: Neo4j Benchmarks
# =============================================================================

class Neo4jBenchmark:
    """Benchmark Neo4j graph database performance"""

    def __init__(self):
        self.driver = GraphDatabase.driver(
            NEO4J_URI,
            auth=(NEO4J_USER, NEO4J_PASSWORD)
        )

    def setup_dataset(self, triple_count: int):
        """Load test data into Neo4j"""
        # Create sample supply chain data
        with self.driver.session() as session:
            # Clear existing data
            session.run("MATCH (n) DETACH DELETE n")

            # Create products and transactions
            for i in range(min(triple_count, 1000)):
                session.run(
                    "CREATE (p:Product {id: 'P' + toString($id)})",
                    id=i
                )

            # Create transactions
            tx_count = triple_count // 2
            for i in range(tx_count):
                session.run(
                    """
                    MATCH (p1:Product {id: 'P' + toString($id1)})
                    CREATE (p1)-[:INvolves]->(t:Transaction {id: 'T' + toString($id2)})
                    """,
                    id1=i % 1000,
                    id2=i
                )

    def benchmark_query(self, query_name: str, query: str) -> List[float]:
        """Run SPARQL-equivalent query and measure latency"""
        # Convert SPARQL to Cypher
        cypher_queries = {
            "simple_select": """
                MATCH (p:Product)-[r:INvolves]->(t:Transaction)
                RETURN p.id, t.id
                LIMIT 10
            """,
            "type_query": """
                MATCH (p:Product)-[r:INvolves]->(t:Transaction)
                RETURN p.id, count(t) AS tx_count
                LIMIT 10
            """,
            "join_query": """
                MATCH (t1:Transaction)-[:INvolves]->(p:Product)
                MATCH (t2:Transaction)-[:INvolves]->(p)
                WHERE t1.id < t2.id
                RETURN t1.id, t2.id
                LIMIT 10
            """,
            "complex_join": """
                MATCH (t1:Transaction)-[:INvolves]->(p1:Product)
                MATCH (t2:Transaction)-[:INvolves]->(p2:Product)
                WHERE p1.supplier = p2.supplier AND t1.id < t2.id
                RETURN t1.id, t2.id, p1.supplier
                LIMIT 10
            """
        }

        cypher = cypher_queries.get(query_name, query)

        latencies = []
        for _ in range(ITERATIONS):
            start = time.perf_counter_ns()
            with self.driver.session() as session:
                result = session.run(cypher)
                list(result)  # Consume all results
            end = time.perf_counter_ns()

            latencies.append((end - start) / 1000)  # Convert to microseconds

        return latencies

    def close(self):
        self.driver.close()


# =============================================================================
# BASELINE 2: Jena Fuseki Benchmarks
# =============================================================================

class JenaBenchmark:
    """Benchmark Apache Jena Fuseki performance"""

    def setup_dataset(self, triple_count: int):
        """Load test data into Jena using SPARQL UPDATE"""
        # Use SPARQL UPDATE to load data
        sparql_update = f"""
            PREFIX : <http://example.org/>

            INSERT DATA {{
        """

        # Add products
        for i in range(min(triple_count, 1000)):
            sparql_update += f" :P{i} a :Product .\n"

        # Add transactions
        for i in range(triple_count // 2):
            sparql_update += f" :T{i} :involves :P{i % 1000} .\n"

        sparql_update += " }\n"

        # Send SPARQL UPDATE
        headers = {"Content-Type": "application/sparql-update"}
        response = requests.post(
            JENA_UPDATE,
            data=sparql_update.strip(),
            headers=headers
        )

        if response.status_code not in [200, 201]:
            print(f"Warning: Jena upload returned {response.status_code}: {response.text[:200]}")

    def benchmark_query(self, query_name: str, query: str) -> List[float]:
        """Run SPARQL query and measure latency"""
        latencies = []

        query = SPARQL_QUERIES.get(query_name, query)

        for i in range(ITERATIONS):
            start = time.perf_counter_ns()

            headers = {"Content-Type": "application/sparql-query"}
            response = requests.post(
                JENA_SPARQL,
                data=query.strip(),
                headers=headers
            )

            end = time.perf_counter_ns()

            if response.status_code == 200:
                latencies.append((end - start) / 1000)
            elif i < 5:  # Only print first few errors
                print(f"Query failed: {response.status_code} - {response.text[:100]}")

        return latencies


# =============================================================================
# BASELINE 3: Ethereum Benchmarks
# =============================================================================

class EthereumBenchmark:
    """Benchmark Ethereum (Ganache) performance"""

    def __init__(self):
        self.w3 = Web3(Web3.HTTPProvider(ETH_RPC))

    def setup_dataset(self, tx_count: int):
        """Deploy test contract and send transactions"""
        # Simple storage contract for testing
        contract_code = """
        contract TestContract {
            uint256 public value;
            event ValueSet(uint256 indexed newValue);

            function setValue(uint256 _value) public {
                value = _value;
                emit ValueSet(_value);
            }
        }
        """

        # This is simplified - in practice you'd compile and deploy
        # For benchmarking, we'll just send raw transactions
        pass

    def benchmark_transaction(self) -> List[float]:
        """Benchmark transaction submission latency"""
        # Check connection
        if not self.w3.is_connected():
            print("Warning: Ethereum node not connected")
            return []

        # Get accounts
        accounts = self.w3.eth.accounts
        if len(accounts) < 2:
            print("Warning: Not enough accounts for Ethereum benchmark")
            return []

        # Get current nonce to avoid conflicts
        current_nonce = self.w3.eth.get_transaction_count(accounts[0])

        latencies = []
        for i in range(min(ITERATIONS, 100)):
            # Build transaction with correct nonce
            tx = {
                'to': accounts[1],
                'from': accounts[0],
                'value': 1,
                'gas': 21000,
                'gasPrice': self.w3.eth.gas_price,
                'nonce': current_nonce + i,
            }

            # Measure transaction submission
            start = time.perf_counter_ns()
            tx_hash = self.w3.eth.send_transaction(tx)
            receipt = self.w3.eth.wait_for_transaction_receipt(tx_hash)
            end = time.perf_counter_ns()

            latencies.append((end - start) / 1_000_000)  # Convert to milliseconds

        return latencies


# =============================================================================
# ProvChainOrg Benchmarks
# =============================================================================

class ProvChainBenchmark:
    """Benchmark native ProvChainOrg performance

    IMPORTANT: ProvChain runs NATIVELY (not in Docker)
    - Use ./scripts/provchain-service.sh start to launch
    - Verify with ./scripts/provchain-service.sh health
    """

    def __init__(self):
        self.auth_token = None
        self._generate_jwt_token()

    def _generate_jwt_token(self) -> None:
        """Generate JWT token for native ProvChainOrg authentication

        Native ProvChain uses development JWT secret by default.
        For production, override with JWT_SECRET environment variable.
        """
        try:
            payload = {
                "sub": "benchmark_runner",
                "exp": int(time.time()) + 3600,  # Expires in 1 hour
                "iat": int(time.time()),
                "iss": "provchain-benchmark"
            }

            self.auth_token = jwt.encode(
                payload,
                PROVCHAIN_JWT_SECRET,
                algorithm="HS256"
            )
            print(f"ProvChain (Native): Generated JWT token for benchmarking")

        except Exception as e:
            print(f"ProvChain (Native): Failed to generate JWT token - {e}")
            print(f"  Ensure PyJWT is installed: pip3 install PyJWT")

    def setup_dataset(self, triple_count: int) -> bool:
        """Load test data into native ProvChainOrg via REST API

        Args:
            triple_count: Number of triples to insert

        Returns:
            True if successful, False otherwise
        """
        if not self.auth_token:
            print("ProvChain (Native): No JWT token available")
            return False

        print(f"ProvChain (Native): Loading {triple_count} triples...")

        try:
            # Insert products (max 1000 for supply chain data)
            product_count = min(triple_count, 1000)
            for i in range(product_count):
                triple = {
                    "subject": f"http://example.org/P{i}",
                    "predicate": "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
                    "object": "http://example.org/Product"
                }
                if not self._insert_triple(triple):
                    print(f"ProvChain (Native): Failed to insert product {i}")
                    return False

            # Insert transaction relationships
            tx_count = triple_count // 2
            for i in range(tx_count):
                triple = {
                    "subject": f"http://example.org/T{i}",
                    "predicate": "http://example.org/involves",
                    "object": f"http://example.org/P{i % product_count}"
                }
                if not self._insert_triple(triple):
                    print(f"ProvChain (Native): Failed to insert transaction {i}")
                    return False

            print(f"ProvChain (Native): Successfully loaded {triple_count} triples")
            return True

        except Exception as e:
            print(f"ProvChain (Native): Error loading dataset - {e}")
            return False

    def _insert_triple(self, triple_dict: Dict[str, str]) -> bool:
        """Insert a single triple via native ProvChain REST API

        Args:
            triple_dict: Dictionary with subject, predicate, object keys

        Returns:
            True if successful, False otherwise
        """
        try:
            response = requests.post(
                f"{PROVCHAIN_URL}/api/transactions",
                json=triple_dict,
                headers={
                    "Authorization": f"Bearer {self.auth_token}",
                    "Content-Type": "application/json"
                },
                timeout=10
            )

            return response.status_code == 200

        except Exception as e:
            if hasattr(self, '_debug') and self._debug:
                print(f"Error inserting triple: {e}")
            return False

    def benchmark_query(self, query_name: str) -> List[float]:
        """Benchmark SPARQL query on native ProvChainOrg

        Args:
            query_name: Name of the query to benchmark

        Returns:
            List of query latencies in microseconds
        """
        if not self.auth_token:
            print("ProvChain (Native): No JWT token - skipping benchmarks")
            return []

        latencies = []
        success_count = 0
        error_count = 0

        query = SPARQL_QUERIES.get(query_name, "")

        if not query:
            print(f"ProvChain (Native): Unknown query '{query_name}'")
            return []

        print(f"ProvChain (Native): Benchmarking '{query_name}' ({ITERATIONS} iterations)...")

        for i in range(ITERATIONS):
            start = time.perf_counter_ns()

            try:
                response = requests.post(
                    f"{PROVCHAIN_URL}/api/sparql/query",
                    json={"query": query},
                    headers={
                        "Authorization": f"Bearer {self.auth_token}",
                        "Content-Type": "application/json"
                    },
                    timeout=30
                )

                end = time.perf_counter_ns()

                if response.status_code == 200:
                    data = response.json()

                    # Validate we got results
                    result_count = data.get("result_count", 0)
                    if result_count > 0:
                        latencies.append((end - start) / 1000)  # Convert to microseconds
                        success_count += 1
                    elif i < 5:  # Only warn for first few iterations
                        print(f"  Warning: Query returned 0 results (iteration {i+1})")
                else:
                    error_count += 1
                    if i < 5:  # Only show first few errors
                        print(f"  Error: HTTP {response.status_code} (iteration {i+1})")

            except requests.exceptions.Timeout:
                error_count += 1
                if i < 5:
                    print(f"  Timeout (iteration {i+1})")

            except requests.exceptions.ConnectionError:
                error_count += 1
                print(f"ProvChain (Native): Connection error - is service running?")
                print(f"  Try: ./scripts/provchain-service.sh status")
                break

            except Exception as e:
                error_count += 1
                if i < 5:
                    print(f"  Exception: {e}")

        # Report results
        print(f"ProvChain (Native): {success_count}/{ITERATIONS} queries successful")

        if success_count < ITERATIONS // 2:
            print(f"ProvChain (Native): WARNING - Less than 50% success rate!")

        if error_count > 0:
            print(f"ProvChain (Native): {error_count} queries failed")

        return latencies


# =============================================================================
# Statistical Analysis
# =============================================================================

def calculate_statistics(latencies: List[float]) -> Optional[Dict]:
    """Calculate descriptive statistics with empty list handling"""
    if not latencies:
        return None

    return {
        "mean": statistics.mean(latencies),
        "median": statistics.median(latencies),
        "stdev": statistics.stdev(latencies) if len(latencies) > 1 else 0,
        "min": min(latencies),
        "max": max(latencies),
        "p50": statistics.median(latencies),
        "p95": statistics.quantiles(latencies, n=20)[18] if len(latencies) > 20 else max(latencies),
        "p99": statistics.quantiles(latencies, n=100)[98] if len(latencies) > 100 else max(latencies),
        "count": len(latencies),
    }


# =============================================================================
# Main Benchmark Runner
# =============================================================================

def run_all_benchmarks():
    """Run baseline comparison benchmarks"""

    results = {
        "neo4j": {},
        "jena": {},
        "ethereum": {},
        "provchain": {}
    }

    # Neo4j Benchmarks
    print("=" * 70)
    print("BASELINE 1: Neo4j Benchmarks")
    print("=" * 70)

    neo4j = Neo4jBenchmark()
    try:
        for size_name, triple_count in BENCHMARK_TRIPLES:
            print(f"\nSetup: Loading {triple_count} triples...")
            neo4j.setup_dataset(triple_count)

            for query_name in SPARQL_QUERIES.keys():
                print(f"Benchmark: {query_name} on {triple_count} triples...")
                latencies = neo4j.benchmark_query(query_name, "")

                stats = calculate_statistics(latencies)
                if stats:
                    key = f"{size_name}_{query_name}"
                    results["neo4j"][key] = stats
                    print(f"  Mean: {stats['mean']:.2f} µs, P95: {stats['p95']:.2f} µs")
                else:
                    print(f"  No valid results")
    finally:
        neo4j.close()

    # Jena Benchmarks
    print("\n" + "=" * 70)
    print("BASELINE 2: Jena Fuseki Benchmarks")
    print("=" * 70)

    jena = JenaBenchmark()
    for size_name, triple_count in BENCHMARK_TRIPLES:
        print(f"\nSetup: Loading {triple_count} triples...")
        jena.setup_dataset(triple_count)

        for query_name in SPARQL_QUERIES.keys():
            print(f"Benchmark: {query_name} on {triple_count} triples...")
            latencies = jena.benchmark_query(query_name, "")

            stats = calculate_statistics(latencies)
            if stats:
                key = f"{size_name}_{query_name}"
                results["jena"][key] = stats
                print(f"  Mean: {stats['mean']:.2f} µs, P95: {stats['p95']:.2f} µs")
            else:
                print(f"  No valid results - skipping")

    # Ethereum Benchmarks
    print("\n" + "=" * 70)
    print("BASELINE 3: Ethereum Benchmarks")
    print("=" * 70)

    eth = EthereumBenchmark()
    print(f"Benchmark: Transaction submission ({ITERATIONS} iterations)...")
    eth.setup_dataset(ITERATIONS)
    latencies = eth.benchmark_transaction()

    stats = calculate_statistics(latencies)
    if stats:
        results["ethereum"]["transaction"] = stats
        print(f"  Mean: {stats['mean']:.2f} ms, P95: {stats['p95']:.2f} ms")
    else:
        print("  No valid results - Ethereum may not be available")

    # ProvChainOrg Benchmarks
    print("\n" + "=" * 70)
    print("SYSTEM UNDER EVALUATION: ProvChainOrg")
    print("=" * 70)

    provchain = ProvChainBenchmark()
    for size_name, triple_count in BENCHMARK_TRIPLES:
        print(f"\nSetup: Loading {triple_count} triples...")
        provchain.setup_dataset(triple_count)

        for query_name in SPARQL_QUERIES.keys():
            print(f"Benchmark: {query_name} on {triple_count} triples...")
            latencies = provchain.benchmark_query(query_name)

            stats = calculate_statistics(latencies)
            if stats:
                key = f"{size_name}_{query_name}"
                results["provchain"][key] = stats
                print(f"  Mean: {stats['mean']:.2f} µs, P95: {stats['p95']:.2f} µs")
            else:
                print(f"  No valid results - ProvChainOrg may not be running")

    # Save results
    os.makedirs(RESULTS_PATH, exist_ok=True)

    with open(f"{RESULTS_PATH}/baseline_comparison.json", 'w') as f:
        json.dump(results, f, indent=2)

    # Generate comparison table
    generate_comparison_table(results)

    print("\n" + "=" * 70)
    print("Benchmarks Complete!")
    print(f"Results saved to: {RESULTS_PATH}/baseline_comparison.json")
    print("=" * 70)


def generate_comparison_table(results: Dict):
    """Generate markdown comparison table for paper"""

    table = """
# Baseline Comparison Results

**Date:** 2026-01-18
**Purpose:** Compare ProvChainOrg against Neo4j, Jena, and Ethereum

## SPARQL Query Performance (Mean Latency in µs)

| System | 100 triples | 1,000 triples | 5,000 triples |
|--------|-------------|---------------|---------------|
"""

    # Extract simple SELECT query results
    for system in ["neo4j", "jena", "provchain"]:
        if system in results:
            row = f"| {system.capitalize()} | "
            for size in ["100", "1000", "5000"]:
                key = f"{size}_simple_select"
                if key in results[system]:
                    row += f"{results[system][key]['mean']:.2f} µs | "
                else:
                    row += "N/A | "
            table += row + "\n"

    # Add Ethereum transaction results
    if "ethereum" in results and "transaction" in results["ethereum"]:
        tx_mean = results["ethereum"]["transaction"]["mean"]
        table += f"\n## Transaction Performance (Mean Latency in ms)\n\n"
        table += f"| System | Mean Latency |\n"
        table += f"|--------|--------------|\n"
        table += f"| Ethereum | {tx_mean:.2f} ms |\n"

    with open(f"{RESULTS_PATH}/COMPARISON_TABLE.md", 'w') as f:
        f.write(table)


if __name__ == "__main__":
    run_all_benchmarks()
