use anyhow::{Context, Result};
use benchmark_runner::{translate_turtle_to_tigergraph, TigerGraphTranslatedDataset};
use clap::Parser;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    input: PathBuf,

    #[arg(long)]
    output_dir: PathBuf,

    #[arg(long, default_value = "ProvChainTrace")]
    graph: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let dataset = translate_turtle_to_tigergraph(&args.input)?;
    fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("failed to create {:?}", args.output_dir))?;

    write_csv(
        args.output_dir.join("products.csv"),
        ["id", "batch_id", "product_type"],
        dataset.products.iter().map(|row| {
            vec![
                row.id.clone(),
                row.batch_id.clone(),
                row.product_type.clone(),
            ]
        }),
    )?;
    write_csv(
        args.output_dir.join("actors.csv"),
        ["id", "actor_type", "name"],
        dataset
            .actors
            .iter()
            .map(|row| vec![row.id.clone(), row.actor_type.clone(), row.name.clone()]),
    )?;
    write_csv(
        args.output_dir.join("transactions.csv"),
        ["id", "quantity", "transaction_date"],
        dataset.transactions.iter().map(|row| {
            vec![
                row.id.clone(),
                row.quantity.clone(),
                row.transaction_date.clone(),
            ]
        }),
    )?;

    for edge_type in ["HAS_PRODUCER", "PROCESSED_BY", "DISTRIBUTED_BY", "SOLD_BY"] {
        let file_name = format!("{}_edges.csv", edge_type.to_ascii_lowercase());
        write_csv(
            args.output_dir.join(file_name),
            ["product_id", "actor_id"],
            dataset
                .product_actor_edges
                .iter()
                .filter(move |row| row.edge_type == edge_type)
                .map(|row| vec![row.product_id.clone(), row.actor_id.clone()]),
        )?;
    }

    write_csv(
        args.output_dir.join("has_transaction_edges.csv"),
        ["product_id", "transaction_id"],
        dataset
            .product_transaction_edges
            .iter()
            .map(|row| vec![row.product_id.clone(), row.transaction_id.clone()]),
    )?;

    for edge_type in ["FROM_ACTOR", "TO_ACTOR"] {
        let file_name = format!("{}_edges.csv", edge_type.to_ascii_lowercase());
        write_csv(
            args.output_dir.join(file_name),
            ["transaction_id", "actor_id"],
            dataset
                .transaction_party_edges
                .iter()
                .filter(move |row| row.edge_type == edge_type)
                .map(|row| vec![row.transaction_id.clone(), row.actor_id.clone()]),
        )?;
    }

    write_csv(
        args.output_dir.join("summary.csv"),
        ["metric", "value"],
        [
            vec!["products".to_string(), dataset.products.len().to_string()],
            vec!["actors".to_string(), dataset.actors.len().to_string()],
            vec![
                "transactions".to_string(),
                dataset.transactions.len().to_string(),
            ],
        ],
    )?;

    let gsql_path = args.output_dir.join("load-and-query.gsql");
    fs::write(&gsql_path, gsql_script(&args.graph, &dataset))?;
    println!(
        "wrote TigerGraph translated artifacts: {}",
        args.output_dir.display()
    );
    println!("products={}", dataset.products.len());
    println!("actors={}", dataset.actors.len());
    println!("transactions={}", dataset.transactions.len());
    println!("gsql={}", gsql_path.display());
    Ok(())
}

fn write_csv<const N: usize, I>(path: PathBuf, header: [&str; N], rows: I) -> Result<()>
where
    I: IntoIterator<Item = Vec<String>>,
{
    let mut file = File::create(&path).with_context(|| format!("failed to create {:?}", path))?;
    writeln!(file, "{}", header.join(","))?;
    for row in rows {
        writeln!(
            file,
            "{}",
            row.iter()
                .map(|value| csv_escape(value))
                .collect::<Vec<_>>()
                .join(",")
        )?;
    }
    Ok(())
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn gsql_script(graph: &str, _dataset: &TigerGraphTranslatedDataset) -> String {
    format!(
        r#"DROP ALL

CREATE VERTEX Product(PRIMARY_ID id STRING, batch_id STRING, product_type STRING) WITH primary_id_as_attribute="true"
CREATE VERTEX Actor(PRIMARY_ID id STRING, actor_type STRING, name STRING) WITH primary_id_as_attribute="true"
CREATE VERTEX Transaction(PRIMARY_ID id STRING, quantity STRING, transaction_date STRING) WITH primary_id_as_attribute="true"
CREATE DIRECTED EDGE HAS_PRODUCER(FROM Product, TO Actor)
CREATE DIRECTED EDGE PROCESSED_BY(FROM Product, TO Actor)
CREATE DIRECTED EDGE DISTRIBUTED_BY(FROM Product, TO Actor)
CREATE DIRECTED EDGE SOLD_BY(FROM Product, TO Actor)
CREATE DIRECTED EDGE HAS_TRANSACTION(FROM Product, TO Transaction)
CREATE DIRECTED EDGE FROM_ACTOR(FROM Transaction, TO Actor)
CREATE DIRECTED EDGE TO_ACTOR(FROM Transaction, TO Actor)
CREATE GRAPH {graph}(Product, Actor, Transaction, HAS_PRODUCER, PROCESSED_BY, DISTRIBUTED_BY, SOLD_BY, HAS_TRANSACTION, FROM_ACTOR, TO_ACTOR)

USE GRAPH {graph}

CREATE LOADING JOB load_trace FOR GRAPH {graph} {{
  DEFINE FILENAME products="/benchmark/tigergraph/generated/products.csv";
  DEFINE FILENAME actors="/benchmark/tigergraph/generated/actors.csv";
  DEFINE FILENAME transactions="/benchmark/tigergraph/generated/transactions.csv";
  DEFINE FILENAME has_producer_edges="/benchmark/tigergraph/generated/has_producer_edges.csv";
  DEFINE FILENAME processed_by_edges="/benchmark/tigergraph/generated/processed_by_edges.csv";
  DEFINE FILENAME distributed_by_edges="/benchmark/tigergraph/generated/distributed_by_edges.csv";
  DEFINE FILENAME sold_by_edges="/benchmark/tigergraph/generated/sold_by_edges.csv";
  DEFINE FILENAME has_transaction_edges="/benchmark/tigergraph/generated/has_transaction_edges.csv";
  DEFINE FILENAME from_actor_edges="/benchmark/tigergraph/generated/from_actor_edges.csv";
  DEFINE FILENAME to_actor_edges="/benchmark/tigergraph/generated/to_actor_edges.csv";

  LOAD products TO VERTEX Product VALUES($0, $1, $2) USING header="true", separator=",";
  LOAD actors TO VERTEX Actor VALUES($0, $1, $2) USING header="true", separator=",";
  LOAD transactions TO VERTEX Transaction VALUES($0, $1, $2) USING header="true", separator=",";
  LOAD has_producer_edges TO EDGE HAS_PRODUCER VALUES($0, $1) USING header="true", separator=",";
  LOAD processed_by_edges TO EDGE PROCESSED_BY VALUES($0, $1) USING header="true", separator=",";
  LOAD distributed_by_edges TO EDGE DISTRIBUTED_BY VALUES($0, $1) USING header="true", separator=",";
  LOAD sold_by_edges TO EDGE SOLD_BY VALUES($0, $1) USING header="true", separator=",";
  LOAD has_transaction_edges TO EDGE HAS_TRANSACTION VALUES($0, $1) USING header="true", separator=",";
  LOAD from_actor_edges TO EDGE FROM_ACTOR VALUES($0, $1) USING header="true", separator=",";
  LOAD to_actor_edges TO EDGE TO_ACTOR VALUES($0, $1) USING header="true", separator=",";
}}

RUN LOADING JOB load_trace

CREATE QUERY product_lookup(STRING product_id) FOR GRAPH {graph} {{
  start = {{Product.*}};
  matched = SELECT p FROM start:p WHERE p.id == product_id;
  PRINT matched;
}}

CREATE QUERY multi_hop_trace(STRING product_id, INT max_hops) FOR GRAPH {graph} {{
  start = {{Product.*}};
  seed = SELECT p FROM start:p WHERE p.id == product_id;
  producers = SELECT a FROM seed:s -(HAS_PRODUCER:e)-> Actor:a;
  processors = SELECT a FROM seed:s -(PROCESSED_BY:e)-> Actor:a;
  distributors = SELECT a FROM seed:s -(DISTRIBUTED_BY:e)-> Actor:a;
  retailers = SELECT a FROM seed:s -(SOLD_BY:e)-> Actor:a;
  transactions = SELECT t FROM seed:s -(HAS_TRANSACTION:e)-> Transaction:t;
  PRINT seed, producers, processors, distributors, retailers, transactions, max_hops;
}}

CREATE QUERY aggregation_by_producer() FOR GRAPH {graph} {{
  producers = {{Actor.*}};
  matched = SELECT p FROM producers:p WHERE p.actor_type == "Producer";
  PRINT matched;
}}

INSTALL QUERY product_lookup
INSTALL QUERY multi_hop_trace
INSTALL QUERY aggregation_by_producer
"#
    )
}
