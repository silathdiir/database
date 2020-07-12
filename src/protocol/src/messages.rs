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

use crate::ColumnMetadata;
use bytes::{Buf, BufMut, BytesMut};

// const PARSE_COMPLETE: u8 = b'1';
// const BIND_COMPLETE: u8 = b'2';
// const CLOSE_COMPLETE: u8 = b'3';
// const NOTIFICATION_RESPONSE: u8 = b'A';
// const COPY_DONE: u8 = b'c';
const COMMAND_COMPLETE: u8 = b'C';
// const COPY_DATA: u8 = b'd';
const DATA_ROW: u8 = b'D';
const ERROR_RESPONSE: u8 = b'E';
const SEVERITY: u8 = b'S';
const CODE: u8 = b'C';
const MESSAGE: u8 = b'M';
// const COPY_IN_RESPONSE: u8 = b'G';
// const COPY_OUT_RESPONSE: u8 = b'H';
const EMPTY_QUERY_RESPONSE: u8 = b'I';
// const BACKEND_KEY_DATA: u8 = b'K';
// const NO_DATA: u8 = b'n';
const NOTICE_RESPONSE: u8 = b'N';
const AUTHENTICATION: u8 = b'R';
// const PORTAL_SUSPENDED: u8 = b's';
const PARAMETER_STATUS: u8 = b'S';
// const PARAMETER_DESCRIPTION: u8 = b't';
const ROW_DESCRIPTION: u8 = b'T';
const READY_FOR_QUERY: u8 = b'Z';

/// Backend PostgreSQL Wire Protocol messages
/// see https://www.postgresql.org/docs/12/protocol-flow.html
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub(crate) enum Message {
    /// A warning message has been issued. The frontend should display the message
    /// but continue listening for ReadyForQuery or ErrorResponse.
    NoticeResponse,
    /// The frontend must now send a PasswordMessage containing the password in
    /// clear-text form. If this is the correct password, the server responds
    /// with an AuthenticationOk, otherwise it responds with an ErrorResponse.
    AuthenticationCleartextPassword,
    /// The frontend must now send a PasswordMessage containing the password
    /// (with user name) encrypted via MD5, then encrypted again using the 4-byte
    /// random salt specified in the AuthenticationMD5Password message. If this
    /// is the correct password, the server responds with an AuthenticationOk,
    /// otherwise it responds with an ErrorResponse. The actual PasswordMessage
    /// can be computed in SQL as concat('md5', md5(concat(md5(concat(password,
    /// username)), random-salt))). (Keep in mind the md5() function returns its
    /// result as a hex string.)
    #[allow(dead_code)]
    AuthenticationMD5Password,
    /// The authentication exchange is successfully completed.
    AuthenticationOk,
    /// Start-up is completed. The frontend can now issue commands.
    ReadyForQuery,
    /// One of the set of rows returned by a SELECT, FETCH, etc query.
    DataRow(Vec<String>),
    /// Indicates that rows are about to be returned in response to a SELECT, FETCH,
    /// etc query. The contents of this message describe the column layout of
    /// the rows. This will be followed by a DataRow message for each row being
    /// returned to the frontend.
    RowDescription(Vec<ColumnMetadata>),
    /// An SQL command completed normally.
    CommandComplete(String),
    /// An empty query string was recognized.
    #[allow(dead_code)]
    EmptyQueryResponse,
    /// An error has occurred. Contains (`Severity`, `Error Code`, `Error Message`)
    /// all of them are optional
    ErrorResponse(Option<&'static str>, Option<&'static str>, Option<String>),
    /// This message informs the frontend about the current (initial) setting of
    /// backend parameters, such as client_encoding or DateStyle
    ///
    /// see https://www.postgresql.org/docs/12/protocol-flow.html#PROTOCOL-ASYNC
    /// 3rd and 4th paragraph
    ParameterStatus(String, String),
}

impl Message {
    /// returns binary representation of a backend message
    pub fn as_vec(&self) -> Vec<u8> {
        match self {
            Message::NoticeResponse => vec![NOTICE_RESPONSE],
            Message::AuthenticationCleartextPassword => vec![AUTHENTICATION, 0, 0, 0, 8, 0, 0, 0, 3],
            Message::AuthenticationMD5Password => vec![AUTHENTICATION, 0, 0, 0, 12, 0, 0, 0, 5, 1, 1, 1, 1],
            Message::AuthenticationOk => vec![AUTHENTICATION, 0, 0, 0, 8, 0, 0, 0, 0],
            Message::ReadyForQuery => vec![READY_FOR_QUERY, 0, 0, 0, 5, EMPTY_QUERY_RESPONSE],
            Message::DataRow(row) => {
                let mut row_buff = BytesMut::with_capacity(256);
                for field in row.iter() {
                    let as_string = field;
                    row_buff.put_i32(as_string.len() as i32);
                    row_buff.extend_from_slice(as_string.as_str().as_bytes());
                }
                let mut len_buff = BytesMut::new();
                len_buff.put_u8(DATA_ROW);
                len_buff.put_i32(6 + row_buff.len() as i32);
                len_buff.put_i16(row.len() as i16);
                len_buff.extend_from_slice(&row_buff);
                len_buff.bytes().to_vec()
            }
            Message::RowDescription(description) => {
                let mut buff = BytesMut::with_capacity(256);
                for field in description.iter() {
                    buff.put_slice(field.name.as_str().as_bytes());
                    buff.put_u8(0); // end of c string
                    buff.put_i32(0); // table id
                    buff.put_i16(0); // column id
                    buff.put_i32(field.type_id);
                    buff.put_i16(field.type_size);
                    buff.put_i32(-1); // type modifier
                    buff.put_i16(0);
                }
                let mut len_buff = BytesMut::new();
                len_buff.put_u8(ROW_DESCRIPTION);
                len_buff.put_i32(6 + buff.len() as i32);
                len_buff.put_i16(description.len() as i16);
                len_buff.extend_from_slice(&buff);
                len_buff.to_vec()
            }
            Message::CommandComplete(command) => {
                let mut command_buff = BytesMut::with_capacity(256);
                command_buff.put_u8(COMMAND_COMPLETE);
                command_buff.put_i32(4 + command.len() as i32 + 1);
                command_buff.extend_from_slice(command.as_bytes());
                command_buff.put_u8(0);
                command_buff.to_vec()
            }
            Message::EmptyQueryResponse => vec![EMPTY_QUERY_RESPONSE, 0, 0, 0, 4],
            Message::ErrorResponse(severity, code, message) => {
                let mut error_response_buff = BytesMut::with_capacity(256);
                error_response_buff.put_u8(ERROR_RESPONSE);
                let mut message_buff = BytesMut::with_capacity(256);
                if let Some(severity) = severity.as_ref() {
                    message_buff.put_u8(SEVERITY);
                    message_buff.extend_from_slice(severity.as_bytes());
                    message_buff.put_u8(0);
                }
                if let Some(code) = code.as_ref() {
                    message_buff.put_u8(CODE);
                    message_buff.extend_from_slice(code.as_bytes());
                    message_buff.put_u8(0);
                }
                if let Some(message) = message.as_ref() {
                    message_buff.put_u8(MESSAGE);
                    message_buff.extend_from_slice(message.as_bytes());
                    message_buff.put_u8(0);
                }
                error_response_buff.put_i32(message_buff.len() as i32 + 4 + 1);
                error_response_buff.extend_from_slice(message_buff.as_ref());
                error_response_buff.put_u8(0);
                error_response_buff.to_vec()
            }
            Message::ParameterStatus(name, value) => {
                let mut parameter_status_buff = BytesMut::with_capacity(256);
                parameter_status_buff.put_u8(PARAMETER_STATUS);
                let mut parameters = BytesMut::with_capacity(256);
                parameters.extend_from_slice(name.as_bytes());
                parameters.put_u8(0);
                parameters.extend_from_slice(value.as_bytes());
                parameters.put_u8(0);
                parameter_status_buff.put_u32(4 + parameters.bytes().len() as u32);
                parameter_status_buff.extend_from_slice(parameters.as_ref());
                parameter_status_buff.to_vec()
            }
        }
    }
}

#[cfg(test)]
mod serialized_messages {
    use super::*;

    #[test]
    fn notice() {
        assert_eq!(Message::NoticeResponse.as_vec(), vec![NOTICE_RESPONSE]);
    }

    #[test]
    fn authentication_cleartext_password() {
        assert_eq!(
            Message::AuthenticationCleartextPassword.as_vec(),
            vec![AUTHENTICATION, 0, 0, 0, 8, 0, 0, 0, 3]
        )
    }

    #[test]
    fn authentication_md5_password() {
        assert_eq!(
            Message::AuthenticationMD5Password.as_vec(),
            vec![AUTHENTICATION, 0, 0, 0, 12, 0, 0, 0, 5, 1, 1, 1, 1]
        )
    }

    #[test]
    fn authentication_ok() {
        assert_eq!(
            Message::AuthenticationOk.as_vec(),
            vec![AUTHENTICATION, 0, 0, 0, 8, 0, 0, 0, 0]
        )
    }

    #[test]
    fn parameter_status() {
        assert_eq!(
            Message::ParameterStatus("client_encoding".to_owned(), "UTF8".to_owned()).as_vec(),
            vec![
                PARAMETER_STATUS,
                0,
                0,
                0,
                25,
                99,
                108,
                105,
                101,
                110,
                116,
                95,
                101,
                110,
                99,
                111,
                100,
                105,
                110,
                103,
                0,
                85,
                84,
                70,
                56,
                0
            ]
        )
    }

    #[test]
    fn ready_for_query() {
        assert_eq!(
            Message::ReadyForQuery.as_vec(),
            vec![READY_FOR_QUERY, 0, 0, 0, 5, EMPTY_QUERY_RESPONSE]
        )
    }

    #[test]
    fn data_row() {
        assert_eq!(
            Message::DataRow(vec!["1".to_owned(), "2".to_owned(), "3".to_owned()]).as_vec(),
            vec![DATA_ROW, 0, 0, 0, 21, 0, 3, 0, 0, 0, 1, 49, 0, 0, 0, 1, 50, 0, 0, 0, 1, 51]
        )
    }

    #[test]
    fn row_description() {
        assert_eq!(
            Message::RowDescription(vec![ColumnMetadata::new("c1".to_owned(), 23, 4)]).as_vec(),
            vec![
                ROW_DESCRIPTION,
                0,
                0,
                0,
                27,
                0,
                1,
                99,
                49,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                23,
                0,
                4,
                255,
                255,
                255,
                255,
                0,
                0
            ]
        );
    }

    #[test]
    fn command_complete() {
        assert_eq!(
            Message::CommandComplete("SELECT".to_owned()).as_vec(),
            vec![COMMAND_COMPLETE, 0, 0, 0, 11, 83, 69, 76, 69, 67, 84, 0]
        )
    }

    #[test]
    fn empty_response() {
        assert_eq!(
            Message::EmptyQueryResponse.as_vec(),
            vec![EMPTY_QUERY_RESPONSE, 0, 0, 0, 4]
        )
    }

    #[test]
    fn error_response() {
        assert_eq!(
            Message::ErrorResponse(None, None, None).as_vec(),
            vec![ERROR_RESPONSE, 0, 0, 0, 5, 0]
        )
    }
}
