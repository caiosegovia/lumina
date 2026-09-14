use crate::{
    catalog,
    models::{
        DiscoveryGroup, DiscoveryIndexResult, DiscoveryItem, DiscoveryOverview, LibraryConfig,
        LocationResolveResult, LocationStatus,
    },
};
use chrono::{Datelike, NaiveDateTime, Utc};
use image::{GenericImageView, ImageReader};
use rusqlite::params;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

const ALGORITHM_VERSION: i64 = 1;

const CITIES: &[(&str, &str, &str, f64, f64)] = &[
    ("São Paulo", "SP", "Brasil", -23.5505, -46.6333),
    ("Rio de Janeiro", "RJ", "Brasil", -22.9068, -43.1729),
    ("Brasília", "DF", "Brasil", -15.7939, -47.8828),
    ("Salvador", "BA", "Brasil", -12.9777, -38.5016),
    ("Fortaleza", "CE", "Brasil", -3.7319, -38.5267),
    ("Belo Horizonte", "MG", "Brasil", -19.9167, -43.9345),
    ("Curitiba", "PR", "Brasil", -25.4284, -49.2733),
    ("Recife", "PE", "Brasil", -8.0476, -34.8770),
    ("Porto Alegre", "RS", "Brasil", -30.0346, -51.2177),
    ("Manaus", "AM", "Brasil", -3.1190, -60.0217),
    ("Belém", "PA", "Brasil", -1.4558, -48.4902),
    ("Goiânia", "GO", "Brasil", -16.6869, -49.2648),
    ("Campinas", "SP", "Brasil", -22.9056, -47.0608),
    ("Santos", "SP", "Brasil", -23.9608, -46.3336),
    ("Florianópolis", "SC", "Brasil", -27.5954, -48.5480),
    ("Vitória", "ES", "Brasil", -20.3155, -40.3128),
    ("Natal", "RN", "Brasil", -5.7793, -35.2009),
    ("Maceió", "AL", "Brasil", -9.6498, -35.7089),
    ("João Pessoa", "PB", "Brasil", -7.1195, -34.8450),
    ("São Luís", "MA", "Brasil", -2.5307, -44.3068),
    ("Lisboa", "Lisboa", "Portugal", 38.7223, -9.1393),
    ("Porto", "Porto", "Portugal", 41.1579, -8.6291),
    ("Nova York", "NY", "Estados Unidos", 40.7128, -74.0060),
    ("Miami", "FL", "Estados Unidos", 25.7617, -80.1918),
    ("Paris", "Île-de-France", "França", 48.8566, 2.3522),
    ("Londres", "Inglaterra", "Reino Unido", 51.5074, -0.1278),
    ("Roma", "Lácio", "Itália", 41.9028, 12.4964),
    (
        "Buenos Aires",
        "Buenos Aires",
        "Argentina",
        -34.6037,
        -58.3816,
    ),
];

fn distance_km(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let r = 6371.0;
    let x = (c - a).to_radians();
    let y = (d - b).to_radians();
    let h = (x / 2.0).sin().powi(2)
        + a.to_radians().cos() * c.to_radians().cos() * (y / 2.0).sin().powi(2);
    2.0 * r * h.sqrt().asin()
}
fn cell_key(lat: f64, lon: f64) -> String {
    format!(
        "{:.2}:{:.2}",
        (lat * 20.0).round() / 20.0,
        (lon * 20.0).round() / 20.0
    )
}

pub fn resolve_locations(cfg: &LibraryConfig) -> Result<LocationResolveResult, String> {
    let mut conn = db(cfg)?;
    let rows = {
        let mut s=conn.prepare("SELECT id,latitude,longitude FROM assets WHERE latitude IS NOT NULL AND longitude IS NOT NULL").map_err(|e|e.to_string())?;
        let values = s
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, f64>(1)?,
                    r.get::<_, f64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        values
    };
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let mut named = 0;
    let mut approximate = 0;
    for (id, lat, lon) in &rows {
        let key = cell_key(*lat, *lon);
        let nearest = CITIES
            .iter()
            .map(|city| (city, distance_km(*lat, *lon, city.3, city.4)))
            .min_by(|a, b| a.1.total_cmp(&b.1));
        let (city, region, country, label, source, precision) = match nearest {
            Some((c, d)) if d <= 100.0 => (
                Some(c.0),
                Some(c.1),
                Some(c.2),
                format!("{}, {} · {}", c.0, c.1, c.2),
                "offline",
                d,
            ),
            _ => (
                None,
                None,
                None,
                format!("Região {:.2}, {:.2}", lat, lon),
                "approximate",
                8.0,
            ),
        };
        if source == "offline" {
            named += 1
        } else {
            approximate += 1
        };
        tx.execute("INSERT INTO location_cells(cell_key,latitude,longitude,city,region,country,display_name,source,precision_km,resolved_at)VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)ON CONFLICT(cell_key)DO NOTHING",params![key,lat,lon,city,region,country,label,source,precision,now]).map_err(|e|e.to_string())?;
        tx.execute("INSERT INTO asset_locations(asset_id,cell_key)VALUES(?1,?2)ON CONFLICT(asset_id)DO UPDATE SET cell_key=excluded.cell_key",params![id,key]).map_err(|e|e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(LocationResolveResult {
        resolved: rows.len() as i64,
        named,
        approximate,
    })
}

pub fn rename_location(cfg: &LibraryConfig, key: &str, name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 120 {
        return Err("Nome do lugar inválido".into());
    }
    let conn = db(cfg)?;
    let affected=conn.execute("INSERT INTO location_overrides(cell_key,display_name,updated_at)SELECT cell_key,?2,?3 FROM location_cells WHERE cell_key=?1 ON CONFLICT(cell_key)DO UPDATE SET display_name=excluded.display_name,updated_at=excluded.updated_at",params![key,name,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
    if affected == 0 {
        return Err("Lugar não encontrado".into());
    }
    Ok(())
}

fn location_status(conn: &rusqlite::Connection) -> Result<LocationStatus, String> {
    conn.query_row("SELECT (SELECT COUNT(*) FROM assets WHERE latitude IS NOT NULL AND longitude IS NOT NULL),(SELECT COUNT(*) FROM asset_locations al JOIN location_cells c ON c.cell_key=al.cell_key WHERE c.source='offline' OR EXISTS(SELECT 1 FROM location_overrides o WHERE o.cell_key=c.cell_key)),(SELECT COUNT(*) FROM asset_locations al JOIN location_cells c ON c.cell_key=al.cell_key WHERE c.source='approximate' AND NOT EXISTS(SELECT 1 FROM location_overrides o WHERE o.cell_key=c.cell_key))",[],|r|Ok(LocationStatus{geotagged:r.get(0)?,named:r.get(1)?,approximate:r.get(2)?})).map_err(|e|e.to_string())
}

fn db(cfg: &LibraryConfig) -> Result<rusqlite::Connection, String> {
    catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())
}

struct VisualFeatures {
    hash: u64,
    width: u32,
    height: u32,
    brightness: f64,
    colorfulness: f64,
    sharpness: f64,
    quality: f64,
    labels: String,
}
fn analyze(path: &Path) -> Result<VisualFeatures, String> {
    let image = ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;
    let (width, height) = image.dimensions();
    let sample = image
        .resize(256, 256, image::imageops::FilterType::Triangle)
        .to_rgb8();
    let mut brightness = 0.0;
    let mut colorfulness = 0.0;
    let mut sharpness = 0.0;
    let mut edges = 0_u64;
    for (x, y, pixel) in sample.enumerate_pixels() {
        let [r, g, b] = pixel.0;
        brightness += (r as f64 + g as f64 + b as f64) / 3.0;
        colorfulness += (r.max(g).max(b) - r.min(g).min(b)) as f64;
        if x > 0 {
            let left = sample.get_pixel(x - 1, y).0;
            sharpness += ((r as i32 - left[0] as i32).abs()
                + (g as i32 - left[1] as i32).abs()
                + (b as i32 - left[2] as i32).abs()) as f64
                / 3.0;
            edges += 1
        }
    }
    let pixels = (sample.width() as u64 * sample.height() as u64).max(1) as f64;
    brightness /= pixels;
    colorfulness /= pixels;
    sharpness /= edges.max(1) as f64;
    let mut labels = Vec::new();
    if width > height * 4 / 3 {
        labels.push("paisagem")
    } else if height > width * 4 / 3 {
        labels.push("retrato")
    } else {
        labels.push("quadrada")
    };
    if brightness < 75.0 {
        labels.push("escura")
    } else if brightness > 180.0 {
        labels.push("clara")
    };
    if colorfulness > 55.0 {
        labels.push("colorida")
    } else {
        labels.push("tons suaves")
    };
    let warmth = sample
        .pixels()
        .map(|p| p[0] as i64 - p[2] as i64)
        .sum::<i64>() as f64
        / pixels;
    if warmth > 12.0 {
        labels.push("quente")
    } else if warmth < -12.0 {
        labels.push("fria")
    };
    let exposure = (1.0 - ((brightness - 127.5).abs() / 127.5)).clamp(0.0, 1.0);
    let resolution = ((width as f64 * height as f64).ln() / 20.0).clamp(0.0, 1.0);
    let quality =
        (resolution * 0.35 + (sharpness / 45.0).clamp(0.0, 1.0) * 0.45 + exposure * 0.20) * 100.0;
    let gray = image
        .resize_exact(9, 8, image::imageops::FilterType::Triangle)
        .to_luma8();
    let mut value = 0_u64;
    for y in 0..8 {
        for x in 0..8 {
            value <<= 1;
            if gray.get_pixel(x, y)[0] > gray.get_pixel(x + 1, y)[0] {
                value |= 1;
            }
        }
    }
    Ok(VisualFeatures {
        hash: value,
        width,
        height,
        brightness,
        colorfulness,
        sharpness,
        quality,
        labels: labels.join(","),
    })
}

pub fn build_index(cfg: &LibraryConfig) -> Result<DiscoveryIndexResult, String> {
    let mut conn = db(cfg)?;
    let rows = {
        let mut stmt = conn.prepare("SELECT a.id,a.master_path FROM assets a LEFT JOIN asset_visual_fingerprints f ON f.asset_id=a.id AND f.algorithm_version=?1 LEFT JOIN asset_visual_traits t ON t.asset_id=a.id AND t.algorithm_version=?1 WHERE a.media_type IN('photo','raw') AND (f.asset_id IS NULL OR t.asset_id IS NULL) ORDER BY a.captured_at DESC").map_err(|e| e.to_string())?;
        let mapped = stmt
            .query_map([ALGORITHM_VERSION], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;
        mapped
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };
    let transaction = conn.transaction().map_err(|e| e.to_string())?;
    let mut result = DiscoveryIndexResult {
        indexed: 0,
        skipped: 0,
        failed: 0,
    };
    for (id, raw_path) in rows {
        let path = PathBuf::from(raw_path);
        if !path.is_file() {
            result.skipped += 1;
            continue;
        }
        match analyze(&path) {
            Ok(features) => {
                let hash = features.hash;
                let bands = [
                    (hash & 0xffff) as i64,
                    ((hash >> 16) & 0xffff) as i64,
                    ((hash >> 32) & 0xffff) as i64,
                    ((hash >> 48) & 0xffff) as i64,
                ];
                transaction.execute("INSERT INTO asset_visual_fingerprints(asset_id,dhash,band0,band1,band2,band3,algorithm_version,indexed_at)VALUES(?1,?2,?3,?4,?5,?6,?7,?8) ON CONFLICT(asset_id) DO UPDATE SET dhash=excluded.dhash,band0=excluded.band0,band1=excluded.band1,band2=excluded.band2,band3=excluded.band3,algorithm_version=excluded.algorithm_version,indexed_at=excluded.indexed_at", params![id,hash as i64,bands[0],bands[1],bands[2],bands[3],ALGORITHM_VERSION,Utc::now().to_rfc3339()]).map_err(|e| e.to_string())?;
                transaction.execute("INSERT INTO asset_visual_traits(asset_id,width,height,brightness,colorfulness,sharpness,quality_score,labels,algorithm_version,indexed_at)VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10) ON CONFLICT(asset_id) DO UPDATE SET width=excluded.width,height=excluded.height,brightness=excluded.brightness,colorfulness=excluded.colorfulness,sharpness=excluded.sharpness,quality_score=excluded.quality_score,labels=excluded.labels,algorithm_version=excluded.algorithm_version,indexed_at=excluded.indexed_at",params![id,features.width,features.height,features.brightness,features.colorfulness,features.sharpness,features.quality,features.labels,ALGORITHM_VERSION,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
                result.indexed += 1;
            }
            Err(_) => result.failed += 1,
        }
    }
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(result)
}

fn item(row: &rusqlite::Row<'_>) -> rusqlite::Result<DiscoveryItem> {
    Ok(DiscoveryItem {
        id: row.get(0)?,
        filename: row.get(1)?,
        media_type: row.get(2)?,
        captured_at: row.get(3)?,
        camera: row.get(4)?,
        quality_score: row.get(5)?,
        visual_labels: row
            .get::<_, Option<String>>(6)?
            .unwrap_or_default()
            .split(',')
            .filter(|x| !x.is_empty())
            .map(str::to_string)
            .collect(),
    })
}

fn all_items(conn: &rusqlite::Connection) -> Result<Vec<DiscoveryItem>, String> {
    let mut stmt=conn.prepare("SELECT a.id,a.filename,a.media_type,a.captured_at,a.camera,t.quality_score,t.labels FROM assets a LEFT JOIN asset_visual_traits t ON t.asset_id=a.id AND t.algorithm_version=1 ORDER BY a.captured_at DESC,a.id").map_err(|e|e.to_string())?;
    let mapped = stmt.query_map([], item).map_err(|e| e.to_string())?;
    mapped
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn parse_date(value: &str) -> Option<chrono::NaiveDateTime> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|x| x.naive_local())
        .ok()
        .or_else(|| NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").ok())
        .or_else(|| {
            NaiveDateTime::parse_from_str(&format!("{value} 00:00:00"), "%Y-%m-%d %H:%M:%S").ok()
        })
}

fn location_groups(
    conn: &rusqlite::Connection,
    by_id: &HashMap<String, DiscoveryItem>,
) -> Result<(Vec<DiscoveryGroup>, Vec<DiscoveryGroup>), String> {
    type PlaceEntry = (String, Option<String>, Option<String>, f64, f64);
    let mut statement = conn
        .prepare("SELECT a.id,a.captured_at,a.latitude,a.longitude,al.cell_key,COALESCE(o.display_name,c.display_name) FROM assets a LEFT JOIN asset_locations al ON al.asset_id=a.id LEFT JOIN location_cells c ON c.cell_key=al.cell_key LEFT JOIN location_overrides o ON o.cell_key=al.cell_key WHERE a.latitude IS NOT NULL AND a.longitude IS NOT NULL ORDER BY a.captured_at,a.id")
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut buckets: HashMap<String, Vec<PlaceEntry>> = HashMap::new();
    for (id, _, latitude, longitude, key, label) in &rows {
        let group_key = key.clone().unwrap_or_else(|| {
            format!(
                "approx:{}:{}",
                (latitude * 10.0).round() as i32,
                (longitude * 10.0).round() as i32
            )
        });
        buckets.entry(group_key).or_default().push((
            id.clone(),
            key.clone(),
            label.clone(),
            *latitude,
            *longitude,
        ));
    }
    let mut places = buckets
        .into_iter()
        .map(|(group_key, entries)| {
            let ids = entries.iter().map(|x| x.0.clone()).collect::<Vec<_>>();
            let count = ids.len();
            DiscoveryGroup {
                id: format!("place-{group_key}"),
                title: entries
                    .iter()
                    .find_map(|x| x.2.clone())
                    .unwrap_or_else(|| format!("Região {:.1}, {:.1}", entries[0].3, entries[0].4)),
                detail: format!("{count} registros com localização"),
                score: count as f64,
                items: ids
                    .into_iter()
                    .filter_map(|id| by_id.get(&id).cloned())
                    .take(12)
                    .collect(),
                recommended_id: None,
                recommendation: Some(
                    "Agrupamento local aproximado; nenhum local foi inferido para arquivos sem GPS"
                        .into(),
                ),
                place_key: entries.iter().find_map(|x| x.1.clone()),
            }
        })
        .collect::<Vec<_>>();
    places.sort_by(|a, b| b.score.total_cmp(&a.score));
    places.truncate(30);

    let mut trip_rows: Vec<(String, NaiveDateTime, f64, f64)> = rows
        .into_iter()
        .filter_map(|(id, date, lat, lon, _, _)| Some((id, parse_date(&date)?, lat, lon)))
        .collect();
    trip_rows.sort_by_key(|row| row.1);
    let mut trips = Vec::new();
    let mut current: Vec<(String, NaiveDateTime, f64, f64)> = Vec::new();
    let flush = |current: &mut Vec<(String, NaiveDateTime, f64, f64)>,
                 trips: &mut Vec<DiscoveryGroup>| {
        if current.len() < 3 {
            current.clear();
            return;
        }
        let taken = std::mem::take(current);
        let center_lat = taken.iter().map(|row| row.2).sum::<f64>() / taken.len() as f64;
        let center_lon = taken.iter().map(|row| row.3).sum::<f64>() / taken.len() as f64;
        let first = taken.first().map(|row| row.1).unwrap();
        let last = taken.last().map(|row| row.1).unwrap();
        trips.push(DiscoveryGroup {
            id: format!("trip-{}", first.format("%Y%m%d")),
            title: format!(
                "Viagem de {} a {}",
                first.format("%d/%m/%Y"),
                last.format("%d/%m/%Y")
            ),
            detail: format!(
                "{} registros · região {:.1}, {:.1}",
                taken.len(),
                center_lat,
                center_lon
            ),
            score: taken.len() as f64,
            items: taken
                .into_iter()
                .filter_map(|row| by_id.get(&row.0).cloned())
                .take(12)
                .collect(),
            recommended_id: None,
            recommendation: Some(
                "Sequência sugerida por datas próximas e coordenadas presentes".into(),
            ),
            place_key: None,
        });
    };
    for row in trip_rows {
        let joins = current
            .last()
            .map(|last| (row.1 - last.1).num_days() <= 3)
            .unwrap_or(true);
        if !joins {
            flush(&mut current, &mut trips);
        }
        current.push(row);
    }
    flush(&mut current, &mut trips);
    trips.sort_by(|a, b| b.id.cmp(&a.id));
    trips.truncate(20);
    Ok((places, trips))
}

pub fn overview(cfg: &LibraryConfig) -> Result<DiscoveryOverview, String> {
    let conn = db(cfg)?;
    let indexable = conn
        .query_row(
            "SELECT COUNT(*) FROM assets WHERE media_type IN('photo','raw')",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let indexed = conn
        .query_row(
            "SELECT COUNT(*) FROM asset_visual_fingerprints f JOIN asset_visual_traits t ON t.asset_id=f.asset_id AND t.algorithm_version=f.algorithm_version WHERE f.algorithm_version=?1",
            [ALGORITHM_VERSION],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let assets = all_items(&conn)?;
    let by_id: HashMap<String, DiscoveryItem> =
        assets.iter().cloned().map(|x| (x.id.clone(), x)).collect();
    let fingerprints = {
        let mut stmt=conn.prepare("SELECT asset_id,dhash,band0,band1,band2,band3 FROM asset_visual_fingerprints WHERE algorithm_version=?1").map_err(|e|e.to_string())?;
        let mapped = stmt
            .query_map([ALGORITHM_VERSION], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)? as u64,
                    [
                        r.get::<_, i64>(2)?,
                        r.get::<_, i64>(3)?,
                        r.get::<_, i64>(4)?,
                        r.get::<_, i64>(5)?,
                    ],
                ))
            })
            .map_err(|e| e.to_string())?;
        mapped
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };
    let mut buckets: HashMap<(usize, i64), Vec<usize>> = HashMap::new();
    for (idx, (_, _, bands)) in fingerprints.iter().enumerate() {
        for (band, value) in bands.iter().enumerate() {
            buckets.entry((band, *value)).or_default().push(idx)
        }
    }
    let mut similar = Vec::new();
    let mut emitted = HashSet::new();
    for (a, (_, hash, bands)) in fingerprints.iter().enumerate() {
        let mut candidates = HashSet::new();
        for (band, value) in bands.iter().enumerate() {
            if let Some(indexes) = buckets.get(&(band, *value)) {
                for candidate in indexes.iter().rev().take(64) {
                    if *candidate != a {
                        candidates.insert(*candidate);
                    }
                }
            }
        }
        let nearest = candidates
            .into_iter()
            .map(|b| (b, (*hash ^ fingerprints[b].1).count_ones()))
            .filter(|(_, distance)| *distance <= 10)
            .min_by_key(|(_, distance)| *distance);
        if let Some((b, distance)) = nearest {
            let pair = (a.min(b), a.max(b));
            if !emitted.insert(pair) {
                continue;
            }
            if let (Some(left), Some(right)) =
                (by_id.get(&fingerprints[a].0), by_id.get(&fingerprints[b].0))
            {
                similar.push(DiscoveryGroup {
                    id: format!("similar-{}-{}", left.id, right.id),
                    title: "Possível variação".into(),
                    detail: format!(
                        "{}% de proximidade visual",
                        100 - (distance as i64 * 100 / 64)
                    ),
                    score: 1.0 - distance as f64 / 64.0,
                    items: vec![left.clone(), right.clone()],
                    recommended_id: [left, right]
                        .into_iter()
                        .max_by(|a, b| {
                            a.quality_score
                                .unwrap_or(0.0)
                                .total_cmp(&b.quality_score.unwrap_or(0.0))
                        })
                        .map(|item| item.id.clone()),
                    recommendation: Some(
                        "Melhor equilíbrio estimado entre detalhe, resolução e luminosidade".into(),
                    ),
                    place_key: None,
                })
            }
        }
    }
    similar.sort_by(|a, b| b.score.total_cmp(&a.score));
    similar.truncate(80);
    let mut sequences = Vec::new();
    let mut current: Vec<DiscoveryItem> = Vec::new();
    let mut chronological = assets.clone();
    chronological.sort_by_key(|x| parse_date(&x.captured_at));
    for asset in chronological {
        let joins = current
            .last()
            .and_then(|last| {
                Some(
                    (parse_date(&asset.captured_at)? - parse_date(&last.captured_at)?)
                        .num_seconds()
                        .abs()
                        <= 15
                        && asset.camera == last.camera,
                )
            })
            .unwrap_or(false);
        if !joins && current.len() >= 3 {
            let taken = std::mem::take(&mut current);
            sequences.push(DiscoveryGroup {
                id: format!("sequence-{}", taken[0].id),
                title: format!("Sequência de {} registros", taken.len()),
                detail: taken[0]
                    .camera
                    .clone()
                    .unwrap_or_else(|| "Mesmo momento".into()),
                score: taken.len() as f64,
                items: taken,
                recommended_id: None,
                recommendation: None,
                place_key: None,
            });
            if let Some(group) = sequences.last_mut() {
                group.recommended_id = group
                    .items
                    .iter()
                    .max_by(|a, b| {
                        a.quality_score
                            .unwrap_or(0.0)
                            .total_cmp(&b.quality_score.unwrap_or(0.0))
                    })
                    .map(|item| item.id.clone());
                group.recommendation =
                    Some("Sugestão técnica; sua escolha continua soberana".into())
            }
        } else if !joins {
            current.clear();
        }
        current.push(asset);
    }
    if current.len() >= 3 {
        sequences.push(DiscoveryGroup {
            id: format!("sequence-{}", current[0].id),
            title: format!("Sequência de {} registros", current.len()),
            detail: current[0]
                .camera
                .clone()
                .unwrap_or_else(|| "Mesmo momento".into()),
            score: current.len() as f64,
            items: current,
            recommended_id: None,
            recommendation: None,
            place_key: None,
        });
        if let Some(group) = sequences.last_mut() {
            group.recommended_id = group
                .items
                .iter()
                .max_by(|a, b| {
                    a.quality_score
                        .unwrap_or(0.0)
                        .total_cmp(&b.quality_score.unwrap_or(0.0))
                })
                .map(|item| item.id.clone());
            group.recommendation = Some("Sugestão técnica; sua escolha continua soberana".into())
        }
    }
    sequences.sort_by(|a, b| b.score.total_cmp(&a.score));
    sequences.truncate(40);
    let now = Utc::now();
    let mut memory_map: HashMap<String, Vec<DiscoveryItem>> = HashMap::new();
    for asset in &assets {
        if let Some(date) = parse_date(&asset.captured_at) {
            if date.year() < now.year() && date.month() == now.month() {
                memory_map
                    .entry(date.format("%Y-%m").to_string())
                    .or_default()
                    .push(asset.clone());
            }
        }
    }
    let mut memories = memory_map
        .into_iter()
        .filter(|(_, items)| !items.is_empty())
        .map(|(key, mut items)| {
            items.truncate(12);
            let year = key.get(0..4).unwrap_or("");
            DiscoveryGroup {
                id: format!("memory-{key}"),
                title: format!("Memórias de {year}"),
                detail: format!("{} registros deste período", items.len()),
                score: items.len() as f64,
                items,
                recommended_id: None,
                recommendation: None,
                place_key: None,
            }
        })
        .collect::<Vec<_>>();
    memories.sort_by(|a, b| b.id.cmp(&a.id));
    memories.truncate(12);
    let (places, trips) = location_groups(&conn, &by_id)?;
    Ok(DiscoveryOverview {
        indexed,
        indexable,
        similar,
        sequences,
        memories,
        places,
        trips,
        location_status: location_status(&conn)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn perceptual_distance_is_stable() {
        assert_eq!((0b1010_u64 ^ 0b1110_u64).count_ones(), 1);
    }

    #[test]
    fn date_parser_accepts_catalog_formats() {
        assert!(parse_date("2025-01-02T03:04:05+00:00").is_some());
        assert!(parse_date("2025-01-02").is_some());
    }

    #[test]
    fn gps_records_form_local_places_and_trips_without_inference() {
        let root = std::env::temp_dir().join(format!("lumina-places-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let cfg = LibraryConfig {
            id: "l".into(),
            name: "Teste".into(),
            master_path: root.to_string_lossy().into(),
            backup_path: root.join("backup").to_string_lossy().into(),
            created_at: Utc::now().to_rfc3339(),
        };
        let conn = db(&cfg).unwrap();
        for (index, date) in ["2026-01-01", "2026-01-02", "2026-01-03"]
            .into_iter()
            .enumerate()
        {
            conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,latitude,longitude,master_path,created_at)VALUES(?1,?2,?3,'photo','jpg',?4,'exif',1,-23.55,-46.63,?3,?4)",params![format!("a{index}"),format!("{:064x}",index+1),format!("a{index}.jpg"),date]).unwrap();
        }
        let items = all_items(&conn).unwrap();
        let by_id = items
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect();
        let (places, trips) = location_groups(&conn, &by_id).unwrap();
        assert_eq!(places[0].score, 3.0);
        assert_eq!(trips[0].items.len(), 3);
        drop(conn);
        let resolved = resolve_locations(&cfg).unwrap();
        assert_eq!(resolved.named, 3);
        let found = overview(&cfg).unwrap();
        assert_eq!(found.places[0].title, "São Paulo, SP · Brasil");
        assert_eq!(found.location_status.named, 3);
        rename_location(&cfg, found.places[0].place_key.as_deref().unwrap(), "Casa").unwrap();
        assert_eq!(overview(&cfg).unwrap().places[0].title, "Casa");
        let resolved_again = resolve_locations(&cfg).unwrap();
        assert_eq!(resolved_again.resolved, 3);
        assert_eq!(overview(&cfg).unwrap().places[0].title, "Casa");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn local_index_finds_a_visual_pair_without_touching_sources() {
        let root = std::env::temp_dir().join(format!("lumina-discovery-{}", Uuid::new_v4()));
        let master = root.join("master");
        let backup = root.join("backup");
        std::fs::create_dir_all(&master).unwrap();
        std::fs::create_dir_all(&backup).unwrap();
        let first = master.join("first.png");
        let second = master.join("second.png");
        for (path, shift) in [(&first, 0_u8), (&second, 1_u8)] {
            let mut image = image::GrayImage::new(90, 80);
            for (x, y, pixel) in image.enumerate_pixels_mut() {
                *pixel = image::Luma([((x * 2 + y + shift as u32) % 255) as u8]);
            }
            image.save(path).unwrap();
        }
        let cfg = LibraryConfig {
            id: "l".into(),
            name: "Teste".into(),
            master_path: master.to_string_lossy().into(),
            backup_path: backup.to_string_lossy().into(),
            created_at: Utc::now().to_rfc3339(),
        };
        let conn = db(&cfg).unwrap();
        for (id, name, path, date) in [
            ("a", "first.png", &first, "2024-09-05"),
            ("b", "second.png", &second, "2024-09-05"),
        ] {
            conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES(?1,?2,?3,'photo','png',?4,'metadata',100,?5,?4)",params![id,id.repeat(64),name,date,path.to_string_lossy()]).unwrap();
        }
        drop(conn);
        let indexed = build_index(&cfg).unwrap();
        assert_eq!(indexed.indexed, 2);
        let found = overview(&cfg).unwrap();
        assert_eq!(found.indexed, 2);
        assert_eq!(found.similar.len(), 1);
        assert!(first.exists() && second.exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
