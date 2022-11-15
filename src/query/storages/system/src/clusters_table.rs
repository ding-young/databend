// Copyright 2021 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;

use common_catalog::table::Table;
use common_catalog::table_context::TableContext;
use common_exception::Result;
use common_expression::types::number::NumberScalar;
use common_expression::types::DataType;
use common_expression::types::NumberDataType;
use common_expression::Chunk;
use common_expression::ColumnBuilder;
use common_expression::DataField;
use common_expression::DataSchemaRefExt;
use common_expression::Scalar;
use common_expression::SchemaDataType;
use common_expression::Value;
use common_meta_app::schema::TableIdent;
use common_meta_app::schema::TableInfo;
use common_meta_app::schema::TableMeta;

use crate::SyncOneBlockSystemTable;
use crate::SyncSystemTable;

pub struct ClustersTable {
    table_info: TableInfo,
}

impl SyncSystemTable for ClustersTable {
    const NAME: &'static str = "system.cluster";

    fn get_table_info(&self) -> &TableInfo {
        &self.table_info
    }

    fn get_full_data(&self, ctx: Arc<dyn TableContext>) -> Result<Chunk> {
        let cluster_nodes = ctx.get_cluster().nodes.clone();

        let mut names = ColumnBuilder::with_capacity(&DataType::String, cluster_nodes.len());
        let mut addresses = ColumnBuilder::with_capacity(&DataType::String, cluster_nodes.len());
        let mut addresses_port = ColumnBuilder::with_capacity(
            &DataType::Number(NumberDataType::UInt16),
            cluster_nodes.len(),
        );
        let mut versions = ColumnBuilder::with_capacity(&DataType::String, cluster_nodes.len());

        for cluster_node in &cluster_nodes {
            let (ip, port) = cluster_node.ip_port()?;

            names.push(Scalar::String(cluster_node.id.as_bytes().to_vec()).as_ref());
            addresses.push(Scalar::String(ip.as_bytes().to_vec()).as_ref());
            addresses_port.push(Scalar::Number(NumberScalar::UInt16(port)).as_ref());
            versions.push(Scalar::String(cluster_node.binary_version.as_bytes().to_vec()).as_ref());
        }

        Ok(Chunk::new(
            vec![
                (Value::Column(names.build()), DataType::String),
                (Value::Column(addresses.build()), DataType::String),
                (
                    Value::Column(addresses_port.build()),
                    DataType::Number(NumberDataType::UInt16),
                ),
                (Value::Column(versions.build()), DataType::String),
            ],
            cluster_nodes.len(),
        ))
    }
}

impl ClustersTable {
    pub fn create(table_id: u64) -> Arc<dyn Table> {
        let schema = DataSchemaRefExt::create(vec![
            DataField::new("name", SchemaDataType::String),
            DataField::new("host", SchemaDataType::String),
            DataField::new("port", SchemaDataType::Number(NumberDataType::UInt16)),
            DataField::new("version", SchemaDataType::String),
        ]);

        let table_info = TableInfo {
            desc: "'system'.'clusters'".to_string(),
            name: "clusters".to_string(),
            ident: TableIdent::new(table_id, 0),
            meta: TableMeta {
                schema,
                engine: "SystemClusters".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        SyncOneBlockSystemTable::create(ClustersTable { table_info })
    }
}
