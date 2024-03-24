mod abi;
mod pb;
use hex_literal::hex;
use pb::eth::erc721::v1 as erc721;
use substreams::{key, prelude::*};
use substreams::{log, store::StoreAddInt64, Hex};
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_database_change::tables::Tables;
use substreams_ethereum::pb::sf::ethereum::r#type::v2 as eth;

#[substreams::handlers::map]
fn graph_out(
    clock: substreams::pb::substreams::Clock,
    transfers: erc721::Transfers,
    owner_deltas: Deltas<DeltaInt64>,
) -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut tables = Tables::new();
    for transfer in transfers.transfers {
        tables
            .create_row(
                "transfer",
                format!("{}-{}", &transfer.trx_hash, transfer.ordinal),
            )
            .set("trx_hash", transfer.trx_hash)
            .set("from", transfer.from)
            .set("to", transfer.to)
            .set("token_id", transfer.token_id)
            .set("ordinal", transfer.ordinal);
    }

    for delta in owner_deltas.into_iter() {
        let holder = key::segment_at(&delta.key, 1);
        let contract = key::segment_at(&delta.key, 2);

        tables
            .create_row("owner_count", format!("{}-{}", contract, holder))
            .set("contract", contract)
            .set("holder", holder)
            .set("balance", delta.new_value)
            .set("block_number", clock.number);
    }

    Ok(tables.to_database_changes())
}

fn generate_key(holder: &String) -> String {
    return format!("total:{}:{}", holder, Hex(TRACKED_CONTRACT));
}
