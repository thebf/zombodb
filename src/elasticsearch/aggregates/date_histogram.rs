use crate::elasticsearch::aggregates::date_histogram::pg_catalog::*;
use crate::elasticsearch::Elasticsearch;
use crate::utils::json_to_string;
use crate::zdbquery::ZDBQuery;
use pgx::*;
use serde::*;
use serde_json::*;

mod pg_catalog {
    use pgx::*;
    use serde::Serialize;

    #[allow(non_camel_case_types)]
    #[derive(PostgresEnum, Serialize)]
    pub(crate) enum CalendarInterval {
        minute,
        hour,
        day,
        week,
        month,
        quarter,
        year,
    }
}

#[pg_extern(immutable, parallel_safe)]
fn date_histogram(
    index: PgRelation,
    field: &str,
    query: ZDBQuery,
    calendar_interval: Option<default!(CalendarInterval, NULL)>,
    fixed_interval: Option<default!(&str, NULL)>,
    time_zone: default!(&str, "+00:00"),
    format: default!(&str, "yyyy-MM-dd"),
) -> impl std::iter::Iterator<
    Item = (
        name!(key_as_string, String),
        name!(term, Option<String>),
        name!(doc_count, i64),
    ),
> {
    #[derive(Deserialize, Serialize)]
    struct BucketEntry {
        key_as_string: String,
        doc_count: i64,
        key: serde_json::Value,
    }

    #[derive(Serialize)]
    struct DateHistogram<'a> {
        field: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        calendar_interval: Option<CalendarInterval>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fixed_interval: Option<&'a str>,
        time_zone: &'a str,
        format: &'a str,
    }

    #[derive(Deserialize, Serialize)]
    struct DateHistogramAggData {
        buckets: Vec<BucketEntry>,
    }

    let elasticsearch = Elasticsearch::new(&index);

    let date_histogram = DateHistogram {
        field,
        calendar_interval,
        fixed_interval,
        time_zone,
        format,
    };

    if date_histogram.calendar_interval.is_some() && date_histogram.fixed_interval.is_some() {
        error!("Both calendar interval and fixed interval have something. Should be mutually exclusive")
    };

    let request = elasticsearch.aggregate::<DateHistogramAggData>(
        query,
        json! {
            {
                "date_histogram":
                    date_histogram
            }
        },
    );

    let result = request
        .execute()
        .expect("failed to execute aggregate search");

    result.buckets.into_iter().map(|entry| {
        (
            entry.key_as_string,
            json_to_string(entry.key),
            entry.doc_count,
        )
    })
}