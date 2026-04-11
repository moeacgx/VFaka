use sea_orm::*;

use aff_common::error::{AppError, AppResult};
use aff_entity::dto::{CreateProductDto, ProductResponse, UpdateProductDto, VariantResponse};
use aff_entity::entities::{card, category, product, product_variant};

pub async fn list_products(
    db: &DatabaseConnection,
    category_id: Option<i32>,
) -> AppResult<Vec<ProductResponse>> {
    let mut query = product::Entity::find()
        .find_also_related(category::Entity)
        .order_by_asc(product::Column::SortOrder);

    if let Some(cid) = category_id {
        query = query.filter(product::Column::CategoryId.eq(cid));
    }

    let results = query
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let product_ids: Vec<i32> = results.iter().map(|(p, _)| p.id).collect();

    let variants = if product_ids.is_empty() {
        vec![]
    } else {
        product_variant::Entity::find()
            .filter(product_variant::Column::ProductId.is_in(product_ids))
            .order_by_asc(product_variant::Column::SortOrder)
            .all(db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
    };

    let mut variant_map: std::collections::HashMap<i32, Vec<VariantResponse>> =
        std::collections::HashMap::new();
    for v in variants {
        variant_map
            .entry(v.product_id)
            .or_default()
            .push(to_variant_response(v));
    }

    Ok(results
        .into_iter()
        .map(|(p, cat)| {
            let vars = variant_map.remove(&p.id).unwrap_or_default();
            to_product_response(p, cat, vars)
        })
        .collect())
}

pub async fn get_product(db: &DatabaseConnection, id: i32) -> AppResult<ProductResponse> {
    let (p, cat) = product::Entity::find_by_id(id)
        .find_also_related(category::Entity)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Product {} not found", id)))?;

    let variants = product_variant::Entity::find()
        .filter(product_variant::Column::ProductId.eq(id))
        .order_by_asc(product_variant::Column::SortOrder)
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let vars = variants.into_iter().map(to_variant_response).collect();
    Ok(to_product_response(p, cat, vars))
}

pub async fn create_product(
    db: &DatabaseConnection,
    dto: CreateProductDto,
) -> AppResult<product::Model> {
    let now = chrono::Utc::now();
    let model = product::ActiveModel {
        category_id: Set(dto.category_id),
        name: Set(dto.name),
        description: Set(dto.description),
        price: Set(dto.price),
        stock_count: Set(0),
        sales_count: Set(0),
        is_active: Set(dto.is_active.unwrap_or(true)),
        allow_alipay: Set(dto.allow_alipay.unwrap_or(true)),
        allow_wxpay: Set(dto.allow_wxpay.unwrap_or(true)),
        allow_qqpay: Set(dto.allow_qqpay.unwrap_or(false)),
        allow_usdt_trc20: Set(dto.allow_usdt_trc20.unwrap_or(false)),
        allow_trx: Set(dto.allow_trx.unwrap_or(false)),
        allow_usdt_erc20: Set(dto.allow_usdt_erc20.unwrap_or(false)),
        post_pay_action_type: Set(dto.post_pay_action_type),
        post_pay_action_value: Set(dto.post_pay_action_value),
        aff_commission_rate: Set(dto.aff_commission_rate),
        sort_order: Set(dto.sort_order.unwrap_or(0)),
        min_quantity: Set(dto.min_quantity.unwrap_or(1)),
        max_quantity: Set(dto.max_quantity.unwrap_or(10)),
        image_url: Set(dto.image_url),
        video_url: Set(dto.video_url),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    product::Entity::insert(model)
        .exec_with_returning(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn update_product(
    db: &DatabaseConnection,
    id: i32,
    dto: UpdateProductDto,
) -> AppResult<product::Model> {
    let existing = product::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Product {} not found", id)))?;

    let mut model: product::ActiveModel = existing.into();

    if let Some(v) = dto.category_id {
        model.category_id = Set(Some(v));
    }
    if let Some(v) = dto.name {
        model.name = Set(v);
    }
    if let Some(v) = dto.description {
        model.description = Set(if v.is_empty() { None } else { Some(v) });
    }
    if let Some(v) = dto.price {
        model.price = Set(v);
    }
    if let Some(v) = dto.is_active {
        model.is_active = Set(v);
    }
    if let Some(v) = dto.allow_alipay {
        model.allow_alipay = Set(v);
    }
    if let Some(v) = dto.allow_wxpay {
        model.allow_wxpay = Set(v);
    }
    if let Some(v) = dto.allow_qqpay {
        model.allow_qqpay = Set(v);
    }
    if let Some(v) = dto.allow_usdt_trc20 {
        model.allow_usdt_trc20 = Set(v);
    }
    if let Some(v) = dto.allow_trx {
        model.allow_trx = Set(v);
    }
    if let Some(v) = dto.allow_usdt_erc20 {
        model.allow_usdt_erc20 = Set(v);
    }
    if let Some(v) = dto.post_pay_action_type {
        model.post_pay_action_type = Set(if v.is_empty() { None } else { Some(v) });
    }
    if let Some(v) = dto.post_pay_action_value {
        model.post_pay_action_value = Set(if v.is_empty() { None } else { Some(v) });
    }
    if let Some(v) = dto.aff_commission_rate {
        model.aff_commission_rate = Set(if v == 0.0 { None } else { Some(v) });
    }
    if let Some(v) = dto.sort_order {
        model.sort_order = Set(v);
    }
    if let Some(v) = dto.min_quantity {
        model.min_quantity = Set(v);
    }
    if let Some(v) = dto.max_quantity {
        model.max_quantity = Set(v);
    }
    if let Some(v) = dto.image_url {
        model.image_url = Set(if v.is_empty() { None } else { Some(v) });
    }
    if let Some(v) = dto.video_url {
        model.video_url = Set(if v.is_empty() { None } else { Some(v) });
    }

    model.updated_at = Set(chrono::Utc::now());

    model
        .update(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn delete_product(db: &DatabaseConnection, id: i32) -> AppResult<()> {
    let existing = product::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Product {} not found", id)))?;

    // Delete all cards associated with this product
    card::Entity::delete_many()
        .filter(card::Column::ProductId.eq(id))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Delete all variants
    product_variant::Entity::delete_many()
        .filter(product_variant::Column::ProductId.eq(id))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let model: product::ActiveModel = existing.into();
    model
        .delete(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

pub async fn batch_delete_products(db: &DatabaseConnection, ids: Vec<i32>) -> AppResult<u64> {
    // Delete all cards for these products
    card::Entity::delete_many()
        .filter(card::Column::ProductId.is_in(ids.clone()))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Delete all variants for these products
    product_variant::Entity::delete_many()
        .filter(product_variant::Column::ProductId.is_in(ids.clone()))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let result = product::Entity::delete_many()
        .filter(product::Column::Id.is_in(ids))
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(result.rows_affected)
}

fn to_variant_response(v: product_variant::Model) -> VariantResponse {
    VariantResponse {
        id: v.id,
        product_id: v.product_id,
        name: v.name,
        price: v.price,
        description: v.description,
        sort_order: v.sort_order,
        is_active: v.is_active,
        stock_count: v.stock_count,
        sales_count: v.sales_count,
        created_at: v.created_at,
        updated_at: v.updated_at,
    }
}

fn to_product_response(
    p: product::Model,
    cat: Option<category::Model>,
    variants: Vec<VariantResponse>,
) -> ProductResponse {
    ProductResponse {
        id: p.id,
        category_id: p.category_id,
        category_name: cat.map(|c| c.name),
        name: p.name,
        description: p.description,
        price: p.price,
        stock_count: p.stock_count,
        sales_count: p.sales_count,
        is_active: p.is_active,
        allow_alipay: p.allow_alipay,
        allow_wxpay: p.allow_wxpay,
        allow_qqpay: p.allow_qqpay,
        allow_usdt_trc20: p.allow_usdt_trc20,
        allow_trx: p.allow_trx,
        allow_usdt_erc20: p.allow_usdt_erc20,
        post_pay_action_type: p.post_pay_action_type,
        post_pay_action_value: p.post_pay_action_value,
        aff_commission_rate: p.aff_commission_rate,
        sort_order: p.sort_order,
        min_quantity: p.min_quantity,
        max_quantity: p.max_quantity,
        image_url: p.image_url,
        video_url: p.video_url,
        created_at: p.created_at,
        updated_at: p.updated_at,
        variants,
    }
}
