/**
 * Server-side API client for the Inventory & Economy bounded context.
 *
 * Routes are nested under /api/v1/inventory on the backend.
 */

import type {
  AddItemRequest,
  CommandResponseWithAggregate,
  EquipItemRequest,
  InventorySummary,
  InventoryView,
  RemoveItemRequest,
} from '$lib/types';

import { apiDelete, apiGet, apiPost } from './client';

const BASE = '/api/v1/inventory';

export async function listInventories(): Promise<InventorySummary[]> {
  return apiGet<InventorySummary[]>(BASE);
}

export async function getInventory(inventoryId: string): Promise<InventoryView> {
  return apiGet<InventoryView>(`${BASE}/${inventoryId}`);
}

export async function addItem(request: AddItemRequest): Promise<CommandResponseWithAggregate> {
  return apiPost<CommandResponseWithAggregate>(`${BASE}/add-item`, request);
}

export async function removeItem(request: RemoveItemRequest): Promise<CommandResponseWithAggregate> {
  return apiPost<CommandResponseWithAggregate>(`${BASE}/remove-item`, request);
}

export async function equipItem(request: EquipItemRequest): Promise<CommandResponseWithAggregate> {
  return apiPost<CommandResponseWithAggregate>(`${BASE}/equip-item`, request);
}

export async function archiveInventory(
  inventoryId: string,
): Promise<CommandResponseWithAggregate> {
  return apiDelete<CommandResponseWithAggregate>(`${BASE}/${inventoryId}`);
}
