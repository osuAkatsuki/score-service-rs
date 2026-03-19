#[derive(Debug)]
pub struct Chart {
    pub chart_id: &'static str,
    pub chart_url: String,
    pub chart_name: &'static str,
    pub entries: Vec<ChartEntry>,
}

#[derive(Debug)]
pub struct ChartEntry {
    pub name: &'static str,
    pub before: Option<String>,
    pub after: String,
}

impl Chart {
    pub fn osu_format(&self) -> String {
        let entries: Vec<_> = self
            .entries
            .iter()
            .map(|entry| entry.osu_format())
            .collect();
        let entries_formatted = entries.join("|");
        format!(
            "chartId:{}|chartUrl:{}|chartName:{}|{entries_formatted}\n",
            self.chart_id, self.chart_url, self.chart_name
        )
    }
}

impl ChartEntry {
    pub fn new_int(name: &'static str, before: Option<i64>, after: i64) -> Self {
        Self {
            name,
            before: before.map(|x| x.to_string()),
            after: after.to_string(),
        }
    }

    pub fn new_float(name: &'static str, before: Option<f32>, after: f32) -> Self {
        fn truncate(value: f32) -> f32 {
            (value * 100.0).trunc() / 100.0
        }

        Self {
            name,
            before: before.map(|x| truncate(x).to_string()),
            after: truncate(after).to_string(),
        }
    }

    pub fn osu_format(&self) -> String {
        match self.before.as_ref() {
            None => format!("{0}Before:|{0}After:{1}", self.name, self.after),
            Some(before) => format!("{0}Before:{1}|{0}After:{2}", self.name, before, self.after),
        }
    }
}
