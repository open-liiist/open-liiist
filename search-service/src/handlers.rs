// search-service/src/handlers.rs

use crate::models::{
    LowestPriceResponse, ProductDB, ProductExistRequest, ProductExistResponse,
    ProductInShopRequest, ProductInShopResponse, ProductResult, ProductsLowestPriceRequest,
    SearchQuery, SearchResponse, ShopProduct, StoreDB, Position, LowestPriceExtended,
};
use crate::search::{
    fetch_lowest_price, fetch_lowest_price_shops, fetch_most_similar, fetch_product_in_shop,
    fetch_product_nearby,
};
use crate::utils::{haversine_distance, sanitize};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::{json}; // removed: Value
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
//use crate::utils::sanitize;


/// Handler per la ricerca dei prodotti: Modalità Standard
pub async fn search_handler(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!("Received search query: {:?}", params);
    let position = Position {
        latitude: params.position_latitude,
        longitude: params.position_longitude,
    };
    tracing::info!("Constructed position: {:?}", position);
    
    // 1. Recupera i prodotti "most similar" (ricerca fuzzy)
    let mut most_similar = match fetch_most_similar(&app_state, &params.query).await {
        Ok(products) => products,
        Err(e) => {
            tracing::error!("Error fetching most similar products: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Internal server error"}))));
        }
    };

    // 2. Escludi gli ID già trovati dalla ricerca per il prezzo più basso
    let exclude_ids: HashSet<String> = most_similar.iter().map(|p| p._id.clone()).collect();

    // 3. Recupera i prodotti per il prezzo più basso
    let mut lowest_price = match fetch_lowest_price(&app_state, &params.query, &exclude_ids, &position).await {
        Ok(products) => products,
        Err(e) => {
            tracing::error!("Error fetching lowest price products: {:?}", e);
            vec![]
        }
    };

    // 4. Se non ci sono risultati per il prezzo più basso, usa il più economico dai "most similar"
    if lowest_price.is_empty() && most_similar.len() > 1 {
        if let Some((_idx, min_product)) = most_similar.iter().enumerate().min_by(|(_, a), (_, b)| {
            a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal)
        }) {
            lowest_price.push(min_product.clone());
            // Se desideri rimuovere l'elemento dai "most similar", ricorda che la variabile è mutabile.
        }
    }

    // 5. Calcola la distanza per ogni prodotto nei "most similar"
    for product in &mut most_similar {
        product.distance = Some(haversine_distance(
            position.latitude,
            position.longitude,
            product.localization.lat,
            product.localization.lon,
        ));
    }

    // 6. Calcola la distanza per ogni prodotto in "lowest price"
    for product in &mut lowest_price {
        product.distance = Some(haversine_distance(
            position.latitude,
            position.longitude,
            product.localization.lat,
            product.localization.lon,
        ));
    }

    // 7. Restituisci la risposta finale
    Ok((StatusCode::OK, Json(SearchResponse { most_similar, lowest_price })))
}

pub async fn check_product_exist(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<ProductExistRequest>,
) -> Result<Json<ProductExistResponse>, (StatusCode, Json<serde_json::Value>)> {
    let products = match fetch_product_nearby(&app_state, &payload.product, payload.position.latitude, payload.position.longitude).await {
        Ok(products) => products,
        Err(e) => {
            tracing::error!("Error fetching product: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Internal server error"}))));
        }
    };

    tracing::info!("Found products: {:#?}", products);
    if let Some(product) = products.first() {
        let distance = haversine_distance(payload.position.latitude, payload.position.longitude, product.localization.lat, product.localization.lon);
        Ok(Json(ProductExistResponse {
            product: payload.product.clone(),
            exists: true,
            details: Some(ProductResult { distance: Some(distance), ..product.clone() }),
        }))
    } else {
        Ok(Json(ProductExistResponse { product: payload.product.clone(), exists: false, details: None }))
    }
}

pub async fn search_product_in_shop(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<ProductInShopRequest>,
) -> Result<Json<ProductInShopResponse>, (StatusCode, Json<serde_json::Value>)> {
    let products = match fetch_product_in_shop(&app_state, &payload.product, &payload.shop, payload.position.latitude, payload.position.longitude).await {
        Ok(products) => products,
        Err(e) => {
            tracing::error!("Error fetching product: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Internal server error"}))));
        }
    };

    tracing::info!("Found products: {:#?}", products);
    if let Some(product) = products.first() {
        let distance = haversine_distance(payload.position.latitude, payload.position.longitude, product.localization.lat, product.localization.lon);
        Ok(Json(ProductInShopResponse {
            product: payload.product.clone(),
            shop: payload.shop.clone(),
            exists: true,
            details: Some(ProductResult { distance: Some(distance), ..product.clone() }),
        }))
    } else {
        Ok(Json(ProductInShopResponse { product: payload.product.clone(), shop: payload.shop.clone(), exists: false, details: None }))
    }
}
// pub async fn find_lowest_price(
//     State(app_state): State<Arc<AppState>>,
//     Json(payload): Json<ProductsLowestPriceRequest>,
// ) -> Result<Json<Vec<LowestPriceResponse>>, (StatusCode, Json<serde_json::Value>)> {
//     tracing::info!("find_lowest_price called with payload: {:?}", payload);

//     let product_prices = match fetch_lowest_price_shops(&app_state, &payload.products, &payload.position).await {
//         Ok(prices) => {
//             tracing::info!("Fetched product prices from Elasticsearch successfully.");
//             prices
//         },
//         Err(e) => {
//             tracing::error!("Error fetching products from Elasticsearch: {:?}", e);
//             return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Internal server error"}))));
//         }
//     };

//     tracing::debug!("Product prices fetched: {:?}", product_prices);

//     // Costruisci la mappa: per ogni negozio, conserva tuple (ShopProduct, input_product)
//     let mut shop_combinations: HashMap<String, Vec<(ShopProduct, String)>> = HashMap::new();
//     for (input_product, product_list) in &product_prices {
//         for product_result in product_list {
//             let distance = haversine_distance(
//                 payload.position.latitude,
//                 payload.position.longitude,
//                 product_result.localization.lat,
//                 product_result.localization.lon,
//             );
//             let sp = ShopProduct {
//                 shop: product_result.localization.grocery.clone(),
//                 name: product_result.name.clone(),
//                 description: product_result.description.clone(),
//                 discount: product_result.discount,
//                 price: product_result.price,
//                 distance,
//             };
//             shop_combinations
//                 .entry(product_result.localization.grocery.clone())
//                 .or_insert_with(Vec::new)
//                 .push((sp, input_product.clone()));
//         }
//     }
//     tracing::info!("Shop combinations built: {:?}", shop_combinations);

//     // Normalizza i termini richiesti
//     let required_names: HashSet<String> = payload.products.iter().map(|p| sanitize(p)).collect();
//     let required_count = required_names.len();
//     tracing::debug!("Required products (sanitized): {:?} (count: {})", required_names, required_count);

//     let mut results: Vec<LowestPriceResponse> = Vec::new();
//     let mode = payload.mode.as_deref().unwrap_or("comodita");
//     tracing::info!("Mode selected: {}", mode);

//     match mode {
//         "risparmio" => {
//             tracing::warn!("La modalità 'risparmio' non è ancora implementata completamente.");
//         },
//         "comodita" => {
//             tracing::info!("Processing in 'comodita' mode.");
//             let mut best_option: Option<LowestPriceResponse> = None;
//             for (shop_name, items) in &shop_combinations {
//                 // Per ogni negozio, seleziona il miglior prodotto per ciascun termine di input
//                 let mut best_for_input: HashMap<String, ShopProduct> = HashMap::new();
//                 for (sp, input_product) in items {
//                     let key = sanitize(input_product);
//                     if let Some(existing) = best_for_input.get(&key) {
//                         if sp.price < existing.price {
//                             best_for_input.insert(key, sp.clone());
//                         }
//                     } else {
//                         best_for_input.insert(key, sp.clone());
//                     }
//                 }
//                 // Se il negozio copre tutti i termini richiesti
//                 if best_for_input.len() == required_count {
//                     let total_price: f64 = best_for_input.values().map(|sp| sp.price).sum();
//                     //tracing::info!("Shop '{}' covers all required products with total price: {}", shop_name, total_price);
//                     tracing::debug!("Shop '{}' - best_for_input: {:?}", shop_name, best_for_input);
//                     let selected_products: Vec<ShopProduct> = best_for_input.into_values().collect();
//                     if let Some(ref mut current_best) = best_option {
//                         if total_price < current_best.total_price {
//                             *current_best = LowestPriceResponse {
//                                 shop: shop_name.clone(),
//                                 total_price,
//                                 products: selected_products,
//                             };
//                         }
//                     } else {
//                         best_option = Some(LowestPriceResponse {
//                             shop: shop_name.clone(),
//                             total_price,
//                             products: selected_products,
//                         });
//                     }
//                 }
//             }
//             if let Some(best) = best_option {
//                 results.push(best);
//             } else {
//                 tracing::warn!("No single shop covers all required products in 'comodita' mode.");
//             }
//         },
//         _ => {
//             tracing::warn!("Unknown mode '{}' received. Defaulting to 'comodita'.", mode);
//         }
//     }
    
//     if results.is_empty() {
//         tracing::info!("No valid shop combinations found to cover all products.");
//     } else {
//         tracing::info!("Returning results: {:?}", results);
//     }
    
//     Ok(Json(results))
// }

pub async fn find_lowest_price(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<ProductsLowestPriceRequest>,
) -> Result<Json<Vec<LowestPriceExtended>>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!("find_lowest_price called with payload: {:?}", payload);

    let product_prices = match fetch_lowest_price_shops(&app_state, &payload.products, &payload.position).await {
        Ok(prices) => {
            tracing::info!("Fetched product prices from Elasticsearch successfully.");
            prices
        },
        Err(e) => {
            tracing::error!("Error fetching products from Elasticsearch: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Internal server error"}))));
        }
    };

    tracing::debug!("Product prices fetched: {:?}", product_prices);

    // Costruisci una mappa per ogni negozio: chiave = nome negozio, valore = Vec<(ShopProduct, input_product)>
    let mut shop_combinations: HashMap<String, Vec<(ShopProduct, String)>> = HashMap::new();
    for (input_product, product_list) in &product_prices {
        for product_result in product_list {
            let distance = haversine_distance(
                payload.position.latitude,
                payload.position.longitude,
                product_result.localization.lat,
                product_result.localization.lon,
            );
            let sp = ShopProduct {
                shop: product_result.localization.grocery.clone(),
                name: product_result.name.clone(),
                description: product_result.description.clone(),
                discount: product_result.discount,
                price: product_result.price,
                distance,
            };
            shop_combinations
                .entry(product_result.localization.grocery.clone())
                .or_insert_with(Vec::new)
                .push((sp, input_product.clone()));
        }
    }
    tracing::info!("Shop combinations built: {:?}", shop_combinations);

    // Insieme dei termini richiesti (sanitizzati)
    let required_names: HashSet<String> = payload.products.iter().map(|p| sanitize(p)).collect();
    let required_count = required_names.len();
    tracing::debug!("Required products (sanitized): {:?} (count: {})", required_names, required_count);

    // Due vettori per raccogliere i negozi che coprono tutti gli item e quelli parziali.
    let mut full_coverage: Vec<LowestPriceExtended> = vec![];
    let mut partial_coverage: Vec<LowestPriceExtended> = vec![];

    // Per ogni negozio in shop_combinations:
    for (shop_name, items) in shop_combinations {
        let mut best_for_input: HashMap<String, ShopProduct> = HashMap::new();
        // Per ogni tuple (ShopProduct, input_product) del negozio, scegli il match migliore (più economico)
        for (sp, input_product) in items {
            let key = sanitize(&input_product);
            best_for_input
                .entry(key)
                .and_modify(|existing| {
                    if sp.price < existing.price {
                        *existing = sp.clone();
                    }
                })
                .or_insert(sp);
        }
        let total_price: f64 = best_for_input.values().map(|sp| sp.price).sum();

        if best_for_input.len() == required_count {
            // Questo negozio copre TUTTI gli item richiesti
            full_coverage.push(LowestPriceExtended {
                shop: shop_name.clone(),
                total_price,
                products: best_for_input.values().cloned().collect(),
                missing: vec![],
            });
        } else {
            // Negozio parziale: calcola quali item mancano
            let found: HashSet<String> = best_for_input.keys().cloned().collect();
            let missing: Vec<String> = required_names.difference(&found).cloned().collect();
            partial_coverage.push(LowestPriceExtended {
                shop: shop_name.clone(),
                total_price,
                products: best_for_input.values().cloned().collect(),
                missing,
            });
        }
    }

    // Determina il negozio "migliore" da restituire
    let mut final_result: Vec<LowestPriceExtended> = vec![];

    if !full_coverage.is_empty() {
        // Caso 1: esistono negozi che coprono tutti gli item: scegli quello con il prezzo totale minore
        let best = full_coverage.into_iter().min_by(|a, b| {
            a.total_price
                .partial_cmp(&b.total_price)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if let Some(best_shop) = best {
            final_result.push(best_shop);
        }
    } else if !partial_coverage.is_empty() {
        // Caso 2: nessun negozio copre tutti gli item; scegli il negozio parziale migliore
        // Criterio: numero di item trovati (maggiore è meglio), in caso di parità, prezzo totale minore.
        let best = partial_coverage.into_iter().max_by(|a, b| {
            let cmp_count = a.products.len().cmp(&b.products.len());
            if cmp_count == std::cmp::Ordering::Equal {
                // Invertiamo l'ordine per il prezzo, perché minore è meglio
                b.total_price.partial_cmp(&a.total_price).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                cmp_count
            }
        });
        if let Some(best_shop) = best {
            tracing::info!("Partial shop '{}' missing items: {:?}", best_shop.shop, best_shop.missing);
            final_result.push(best_shop);
        }
    } else {
        tracing::info!("No valid shop combinations found to cover any products.");
    }

    tracing::info!("Returning results: {:?}", final_result);
    Ok(Json(final_result))
}

pub async fn get_all_stores(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Vec<StoreDB>>, axum::http::StatusCode> {
    let db_pool = &app_state.db_pool;
    let stores = sqlx::query_as::<_, StoreDB>(
        r#"
        SELECT id, grocery, lat, lng, street, city, zip_code, working_hours, picks_up_in_store
        FROM "Localization"
        "#
    )
    .fetch_all(db_pool)
    .await
    .map_err(|e| {
        eprintln!("Database query failed: {:?}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(stores))
}

pub async fn get_products_by_store(
    Path(store_id): Path<i32>,
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Vec<ProductDB>>, axum::http::StatusCode> {
    let db_pool = &app_state.db_pool;
    let products = sqlx::query_as::<_, ProductDB>(
        r#"
        SELECT p.id, p.name, p.description, p.current_price, p.discount, p.price_for_kg, p.image_url
        FROM "Product" p
        WHERE p."localizationId" = $1
        "#
    )
    .bind(store_id)
    .fetch_all(db_pool)
    .await
    .map_err(|e| {
        eprintln!("Database query failed: {:?}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(products))
}

