use mcp_sdk::schemas::tool_schema::{ToolInputProperty, ToolInputSchema, ToolInputType};

use crate::schemas::{
    cells::matrix::{MAX_MUTATION_COLUMNS, MAX_MUTATION_ROWS},
    requests::sheets_mutations::MAX_CREATE_SPREADSHEET_GRID_DIMENSION,
};

const EMPTY_CELL_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &["kind"],
    &[ToolInputProperty::string_enum("kind", &["empty"])],
);
const TEXT_CELL_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &["kind", "value"],
    &[
        ToolInputProperty::string_enum("kind", &["text"]),
        ToolInputProperty::string("value", None, Some(64 * 1024)),
    ],
);
const NUMBER_CELL_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &["kind", "value"],
    &[
        ToolInputProperty::string_enum("kind", &["number"]),
        ToolInputProperty::number("value", None, None),
    ],
);
const BOOLEAN_CELL_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &["kind", "value"],
    &[
        ToolInputProperty::string_enum("kind", &["boolean"]),
        ToolInputProperty::boolean("value"),
    ],
);
const FORMULA_CELL_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &["kind", "value"],
    &[
        ToolInputProperty::string_enum("kind", &["formula"]),
        ToolInputProperty::string("value", Some(1), Some(64 * 1024)),
    ],
);

const CELL_VARIANTS: &[ToolInputType] = &[
    ToolInputType::object(&EMPTY_CELL_SCHEMA),
    ToolInputType::object(&TEXT_CELL_SCHEMA),
    ToolInputType::object(&NUMBER_CELL_SCHEMA),
    ToolInputType::object(&BOOLEAN_CELL_SCHEMA),
    ToolInputType::object(&FORMULA_CELL_SCHEMA),
];
const CELL_TYPE: ToolInputType = ToolInputType::one_of(CELL_VARIANTS);
const ROW_TYPE: ToolInputType =
    ToolInputType::array(Some(1), Some(MAX_MUTATION_COLUMNS), &CELL_TYPE);
const INITIAL_TAB_SCHEMA: ToolInputSchema = ToolInputSchema::object(
    &[],
    &[
        ToolInputProperty::string("title", Some(1), Some(256)),
        ToolInputProperty::integer(
            "row_count",
            Some(1),
            Some(MAX_CREATE_SPREADSHEET_GRID_DIMENSION as u64),
        ),
        ToolInputProperty::integer(
            "column_count",
            Some(1),
            Some(MAX_CREATE_SPREADSHEET_GRID_DIMENSION as u64),
        ),
        ToolInputProperty::integer(
            "frozen_row_count",
            Some(1),
            Some(MAX_CREATE_SPREADSHEET_GRID_DIMENSION as u64),
        ),
        ToolInputProperty::integer(
            "frozen_column_count",
            Some(1),
            Some(MAX_CREATE_SPREADSHEET_GRID_DIMENSION as u64),
        ),
    ],
);

pub(crate) const INITIAL_TAB_PROPERTY: ToolInputProperty =
    ToolInputProperty::object("initial_tab", &INITIAL_TAB_SCHEMA);

pub(crate) const fn cell_matrix_property(name: &'static str) -> ToolInputProperty {
    ToolInputProperty::array(name, Some(1), Some(MAX_MUTATION_ROWS), &ROW_TYPE)
}

pub(crate) const fn cell_rows_property(name: &'static str) -> ToolInputProperty {
    ToolInputProperty::array(name, Some(1), Some(MAX_MUTATION_ROWS), &ROW_TYPE)
}
