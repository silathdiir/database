// Copyright 2020 Alex Dukhno
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

use data_manager::DataManager;
use plan::TableCreationInfo;
use std::sync::Arc;

pub(crate) struct CreateTableCommand {
    table_info: TableCreationInfo,
    data_manager: Arc<DataManager>,
}

impl CreateTableCommand {
    pub(crate) fn new(table_info: TableCreationInfo, data_manager: Arc<DataManager>) -> CreateTableCommand {
        CreateTableCommand {
            table_info,
            data_manager,
        }
    }

    pub(crate) fn execute(&mut self) {
        let (schema_id, table_name, columns) = self.table_info.as_tuple();
        if let Err(()) = self.data_manager.create_table(schema_id, table_name, columns) {
            log::error!("Error while creating a table {}.{}", schema_id, table_name);
        }
    }
}
