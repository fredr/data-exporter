# data-exporter
A prometheus exporter that scrapes remote data or local files and converts them to prometheus metrics. It is similar to [json_exporter](https://github.com/prometheus-community/json_exporter/), but adds the possibility to transform the data before metrics are extracted, and is not limited to only support JSON data. 

## Local development
### Run tests
```
cargo test
```
### Run the exporter
```
cargo run -- --config examples/config.yaml
```

## Configuration
data-exporter is configured via a configuration file in YAML format (see [this example](https://github.com/fredr/data-exporter/blob/main/examples/config.yaml)), and via command-line flags.

### Command-line flags
Run the help command to get all available flags
```
data-exporter --help
```

### Configuration file
In the configuration file all scraped metrics are configured.o

```
metrics: [<metric_config>]
```

### <metric_config>
```
# name of the metric when scraped
name: <string>

# metric help string when scraped
help: <string>

# targets to scrape data from
targets: [<target_config>]

# pipeline stages to transform the data before parsing it
pipeline_stages: [<pipeline_stage_config>]

# parser for parsing metrics from data
parser: <parser_config>

# set a constant value for the metric, it is required to set either this or `value` in `parser_config`
value: <float64>
```

### <target_config>
#### file
```
type: file

# path to local file
path: <string>
```
#### http
```
type: http

# url to scrape
url: <string>
```

### <pipeline_stage_config>
#### jq
```
type: jq

# jq query to execute on data
query: <string>
```

#### regex
```
type: regex

# regex pattern
pattern: <regex>

# replacement string
replace: <string>
```

### <parser_config>
#### json
```
type: json

# fields to extract as labels
labels: [<string>]

# field to extract as value, it is required to set either this or `value` in `metric_config`
value: <string>
```
