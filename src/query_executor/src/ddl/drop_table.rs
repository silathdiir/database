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
use plan::FullTableId;
use std::sync::Arc;

pub(crate) struct DropTableCommand {
    table_id: FullTableId,
    data_manager: Arc<DataManager>,
}

impl DropTableCommand {
    pub(crate) fn new(table_id: FullTableId, data_manager: Arc<DataManager>) -> DropTableCommand {
        DropTableCommand { table_id, data_manager }
    }

    pub(crate) fn execute(&mut self) {
        if let Err(()) = self.data_manager.drop_table(&self.table_id) {
            log::error!("Error while dropping table {:?}", self.table_id);
        }
    }
}
