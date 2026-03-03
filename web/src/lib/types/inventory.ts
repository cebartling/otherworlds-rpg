/**
 * Types for the Inventory bounded context.
 *
 * Corresponds to Rust structs in:
 * - otherworlds-api/src/routes/inventory.rs (requests)
 * - otherworlds-inventory/src/application/query_handlers.rs (views)
 */

import type { UUID } from './common';

// ---------------------------------------------------------------------------
// Command / request types
// ---------------------------------------------------------------------------

/** Request body for POST /api/v1/inventory/add-item. */
export interface AddItemRequest {
  inventory_id: UUID;
  item_id: UUID;
}

/** Request body for POST /api/v1/inventory/remove-item. */
export interface RemoveItemRequest {
  inventory_id: UUID;
  item_id: UUID;
}

/** Request body for POST /api/v1/inventory/equip-item. */
export interface EquipItemRequest {
  inventory_id: UUID;
  item_id: UUID;
}

// ---------------------------------------------------------------------------
// Query / view types
// ---------------------------------------------------------------------------

/** Full read-only view of an inventory (GET /api/v1/inventory/:id). */
export interface InventoryView {
  inventory_id: UUID;
  items: UUID[];
  version: number;
}

/** Summary view for listing inventories (GET /api/v1/inventory). */
export interface InventorySummary {
  inventory_id: UUID;
  item_count: number;
  version: number;
}
