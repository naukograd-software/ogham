#!/usr/bin/env python3
"""Validate the golden example IR against expected language constructs.

Categorizes failures as:
  ERROR       — wrong data, indicates a compiler bug or schema issue
  KNOWN BUG   — known compiler limitation, tracked separately
"""

import json
import sys
from pathlib import Path

ir = json.load(open(Path(__file__).parent / "ir.json"))

types_by_name = {t["fullName"]: t for t in ir["types"]}
enums_by_name = {e["fullName"]: e for e in ir["enums"]}
services_by_name = {s["fullName"]: s for s in ir["services"]}

errors = []
known_bugs = []


def error(msg):
    errors.append(msg)


def known_bug(msg):
    known_bugs.append(msg)


def get_type(full_name):
    t = types_by_name.get(full_name)
    if t is None:
        error(f"type {full_name} not found in IR")
    return t


def get_enum(full_name):
    e = enums_by_name.get(full_name)
    if e is None:
        error(f"enum {full_name} not found in IR")
    return e


def get_service(full_name):
    s = services_by_name.get(full_name)
    if s is None:
        error(f"service {full_name} not found in IR")
    return s


def field_names(t):
    if t is None:
        return []
    return [f["name"] for f in t.get("fields", [])]


def field_by_name(t, name):
    if t is None:
        return None
    for f in t.get("fields", []):
        if f["name"] == name:
            return f
    return None


def field_type_full(f):
    """Extract type info from a field."""
    if f is None:
        return None
    ty = f.get("type", {})
    if "messageType" in ty:
        return ty["messageType"].get("fullName")
    if "scalar" in ty:
        return "scalar:" + ty["scalar"].get("scalarKind", "?")
    if "enumType" in ty:
        return ty["enumType"].get("fullName")
    if "map" in ty:
        return "map"
    return None


def has_map_type(f):
    if f is None:
        return False
    return "map" in f.get("type", {})


def oneof_names(t):
    if t is None:
        return []
    return [o["name"] for o in t.get("oneofs", [])]


def oneof_by_name(t, name):
    if t is None:
        return None
    for o in t.get("oneofs", []):
        if o["name"] == name:
            return o
    return None


def enum_value_names(e):
    if e is None:
        return []
    return [v["name"] for v in e.get("values", [])]


def rpc_by_name(s, name):
    if s is None:
        return None
    for r in s.get("rpcs", []):
        if r["name"] == name:
            return r
    return None


def field_annotations(f):
    if f is None:
        return []
    return [a["name"] for a in f.get("annotations", [])]


def type_annotations(t):
    if t is None:
        return []
    return [a["name"] for a in t.get("annotations", [])]


def check_field_has_annotation(t, field_name, ann_name):
    f = field_by_name(t, field_name)
    if f is None:
        error(f"{t['fullName']}.{field_name}: field not found")
        return
    if ann_name not in field_annotations(f):
        error(f"{t['fullName']}.{field_name}: missing annotation @{ann_name}")


# ============================================================================
# 1. All expected types exist
# ============================================================================
print("--- 1. Types exist ---")

for name in [
    # common
    "common.Id", "common.TrackingNumber", "common.WeightKg",
    "common.IsFragile", "common.Barcode",
    "common.Address", "common.ContactInfo", "common.Cost",
    "common.TimeWindow", "common.Dimensions", "common.Metadata",
    "common.SystemLimits",
    # fleet
    "fleet.Vehicle", "fleet.Driver", "fleet.GpsPosition",
    "fleet.DriverAssignment", "fleet.Vehicle.MaintenanceRecord",
    # warehouse
    "warehouse.Warehouse", "warehouse.InventoryItem",
    "warehouse.StockMovement", "warehouse.Warehouse.Zone",
    # shipment
    "shipment.Package", "shipment.Shipment",
    "shipment.StandardDelivery", "shipment.ExpressDelivery",
    "shipment.ScheduledDelivery", "shipment.TrackingEvent",
    "shipment.ShipmentMetrics",
    # shipment projections
    "shipment.ShipmentSummary", "shipment.ShipmentFlatRow",
    "shipment.DeliveryCostView", "shipment.DeliveryFlat",
    "shipment.ShipmentDashboard", "shipment.ShipmentSearch",
    "shipment.ShipmentCard",
    # shipment Pick/Omit
    "shipment.ShipmentPublic", "shipment.ShipmentWithoutCosts",
    "shipment.ShipmentNoAudit",
    # order
    "order.Order", "order.LineItem", "order.CardPayment",
    "order.BankPayment", "order.OrderPaymentFlat", "order.OrderRow",
    "order.PaginatedOrders", "order.OrderPublic", "order.OrderWithoutPayment",
    # golden (api)
    "golden.ListResponse",
    # generic expansions
    "golden.ListResponseVehicle", "golden.ListResponseInventoryItem",
    "golden.ListResponseShipmentSummary", "golden.ListResponseOrderPublic",
]:
    get_type(name)

print("--- 1b. Enums exist ---")
for name in [
    "fleet.VehicleType", "fleet.VehicleStatus", "fleet.DriverLicense",
    "fleet.Driver.EmploymentStatus",
    "warehouse.InventoryStatus", "warehouse.MovementType",
    "warehouse.Warehouse.Zone.ZoneTemperature",
    "shipment.ShipmentStatus", "shipment.PackageSize",
    "order.OrderStatus", "order.PaymentMethod",
]:
    get_enum(name)

print("--- 1c. Services exist ---")
for name in ["golden.FleetService", "golden.WarehouseService",
             "golden.ShipmentService", "golden.OrderService"]:
    get_service(name)

# ============================================================================
# 2. All 16 primitive types in SystemLimits
# ============================================================================
print("--- 2. Primitive types (SystemLimits) ---")

sl = get_type("common.SystemLimits")
if sl:
    expected = [
        "enabled", "region", "encryption_key", "priority_level",
        "retry_count", "batch_size", "max_payload_bytes", "sequence_number",
        "flags", "header_version", "port", "ttl_seconds",
        "total_processed", "counter", "temperature_celsius", "precise_weight",
    ]
    actual = field_names(sl)
    if actual != expected:
        error(f"SystemLimits fields mismatch:\n  expected: {expected}\n  actual:   {actual}")

# ============================================================================
# 3. Shape injection — BaseModel(1..4) expands correctly
# ============================================================================
print("--- 3. Shape injection ---")

vehicle = get_type("fleet.Vehicle")
if vehicle:
    for i, name in enumerate(["id", "created_at", "updated_at", "deleted_at"]):
        f = vehicle["fields"][i]
        if f["name"] != name:
            error(f"Vehicle field {i}: expected '{name}', got '{f['name']}'")
        trace = f.get("trace", {}).get("shape", {})
        if trace.get("shapeName") != "BaseModel":
            error(f"Vehicle.{name}: shape trace should be BaseModel, got {trace}")
        if trace.get("injectionRangeStart") != 1 or trace.get("injectionRangeEnd") != 4:
            error(f"Vehicle.{name}: wrong injection range")

# ============================================================================
# 4. @default::Default(now) propagated from shape
# ============================================================================
print("--- 4. @default::Default(now) ---")

if vehicle:
    check_field_has_annotation(vehicle, "created_at", "Default")
    check_field_has_annotation(vehicle, "updated_at", "Default")

# ============================================================================
# 5. Type aliases (0 fields)
# ============================================================================
print("--- 5. Type aliases ---")

for alias in ["common.Id", "common.TrackingNumber", "common.WeightKg",
              "common.IsFragile", "common.Barcode"]:
    t = get_type(alias)
    if t and len(t.get("fields", [])) != 0:
        error(f"{alias}: type alias should have 0 fields")

# ============================================================================
# 6. Enums: implicit Unspecified
# ============================================================================
print("--- 6. Enums ---")

vt = get_enum("fleet.VehicleType")
if vt:
    names = enum_value_names(vt)
    if "Unspecified" not in names:
        error("VehicleType: missing implicit Unspecified=0")
    for v in ["Van", "Truck", "Motorcycle", "Bicycle",
              "SemiTrailer", "BoxTruck"]:
        if v not in names:
            error(f"VehicleType: missing value {v}")

# ============================================================================
# 7. Nested types and enums
# ============================================================================
print("--- 7. Nested types/enums ---")

if vehicle:
    nested = [nt["name"] for nt in vehicle.get("nestedTypes", [])]
    if "MaintenanceRecord" not in nested:
        error("Vehicle: missing nested type MaintenanceRecord")

driver = get_type("fleet.Driver")
if driver:
    nested_enums = [ne["name"] for ne in driver.get("nestedEnums", [])]
    if "EmploymentStatus" not in nested_enums:
        error("Driver: missing nested enum EmploymentStatus")

wh = get_type("warehouse.Warehouse")
if wh:
    nested = [nt["name"] for nt in wh.get("nestedTypes", [])]
    if "Zone" not in nested:
        error("Warehouse: missing nested type Zone")

zone = get_type("warehouse.Warehouse.Zone")
if zone:
    nested_enums = [ne["name"] for ne in zone.get("nestedEnums", [])]
    if "ZoneTemperature" not in nested_enums:
        error("Zone: missing nested enum ZoneTemperature")

# ============================================================================
# 8. Oneofs
# ============================================================================
print("--- 8. Oneofs ---")

if driver:
    if "location" not in oneof_names(driver):
        error("Driver: missing oneof 'location'")
    loc = oneof_by_name(driver, "location")
    if loc:
        fnames = [f["name"] for f in loc.get("fields", [])]
        if "depot" not in fnames or "live" not in fnames:
            error(f"Driver.location: expected depot+live, got {fnames}")

shipment_t = get_type("shipment.Shipment")
if shipment_t:
    if "delivery_method" not in oneof_names(shipment_t):
        error("Shipment: missing oneof 'delivery_method'")

order_t = get_type("order.Order")
if order_t:
    if "payment_details" not in oneof_names(order_t):
        error("Order: missing oneof 'payment_details'")

# ============================================================================
# 9. Pick / Omit
# ============================================================================
print("--- 9. Pick/Omit ---")

# Pick<Shipment, tracking_number, status, destination, receiver>
sp = get_type("shipment.ShipmentPublic")
if sp:
    actual = set(field_names(sp))
    expected = {"tracking_number", "status", "destination", "receiver"}
    if actual != expected:
        error(f"ShipmentPublic fields: expected {expected}, got {actual}")

# Omit<Shipment, shipping_cost, insurance_cost>
swc = get_type("shipment.ShipmentWithoutCosts")
if swc and shipment_t:
    removed = set(field_names(shipment_t)) - set(field_names(swc))
    for f in ["shipping_cost", "insurance_cost"]:
        if f not in removed:
            error(f"ShipmentWithoutCosts: '{f}' should be removed, diff={removed}")

# Omit<Shipment, common.AuditFields> — KNOWN BUG: qualified shape in Omit not resolved
sna = get_type("shipment.ShipmentNoAudit")
if sna and shipment_t:
    removed = set(field_names(shipment_t)) - set(field_names(sna))
    if "created_at" not in removed or "updated_at" not in removed:
        known_bug("Omit<Shipment, common.AuditFields>: qualified shape name not resolved in Omit — "
                   "created_at/updated_at not removed")

# OrderPublic = Pick<Order, order_number, status, total, tracking_number>
op = get_type("order.OrderPublic")
if op:
    actual = set(field_names(op))
    expected = {"order_number", "status", "total", "tracking_number"}
    if actual != expected:
        error(f"OrderPublic fields: expected {expected}, got {actual}")

# OrderWithoutPayment = Omit<Order, payment_method, payment_details>
owp = get_type("order.OrderWithoutPayment")
if owp and order_t:
    removed = set(field_names(order_t)) - set(field_names(owp))
    if "payment_method" not in removed:
        error(f"OrderWithoutPayment: 'payment_method' should be removed, diff={removed}")

# ============================================================================
# 10. Projections (field mappings)
# ============================================================================
print("--- 10. Projections ---")

def check_mapping(type_name, field_name):
    t = get_type(type_name)
    if t is None:
        return
    f = field_by_name(t, field_name)
    if f is None:
        error(f"{type_name}.{field_name}: field not found")
        return
    if f.get("mapping") is None:
        error(f"{type_name}.{field_name}: missing projection mapping")

# Simple
check_mapping("shipment.ShipmentSummary", "cost")
check_mapping("shipment.ShipmentSummary", "tracking_number")

# Nested flatten
check_mapping("shipment.ShipmentFlatRow", "origin_city")
check_mapping("shipment.ShipmentFlatRow", "dest_country")

# Multi-source
check_mapping("shipment.ShipmentDashboard", "distance_km")
check_mapping("shipment.ShipmentDashboard", "receiver_name")

# Chain
check_mapping("shipment.ShipmentCard", "tracking_number")

# Wildcard
check_mapping("order.OrderPaymentFlat", "transaction_id")

# No mapping on new fields
ss_search = get_type("shipment.ShipmentSearch")
if ss_search:
    f = field_by_name(ss_search, "search_text")
    if f and f.get("mapping") is not None:
        error("ShipmentSearch.search_text: should have no mapping (new field)")

# ============================================================================
# 11. Annotations
# ============================================================================
print("--- 11. Annotations ---")

# Field-level annotations (these work)
if vehicle:
    check_field_has_annotation(vehicle, "plate_number", "Length")
if driver:
    check_field_has_annotation(driver, "first_name", "Required")
    check_field_has_annotation(driver, "first_name", "Length")
    check_field_has_annotation(driver, "phone", "Pattern")

inv = get_type("warehouse.InventoryItem")
if inv:
    check_field_has_annotation(inv, "sku", "NotEmpty")
    check_field_has_annotation(inv, "quantity", "Range")

# Type-level custom annotations — KNOWN BUG: not lowered to IR
for tname, ann in [
    ("warehouse.Warehouse", "Table"),
    ("warehouse.InventoryItem", "Table"),
    ("order.Order", "Table"),
    ("order.Order", "Pipeline"),
]:
    t = get_type(tname)
    if t and ann not in type_annotations(t):
        known_bug(f"{tname}: type-level @{ann} not in IR (custom annotations on types not lowered)")

# @common::Column on fields — KNOWN BUG
if inv:
    f = field_by_name(inv, "sku")
    if f and "Column" not in field_annotations(f):
        known_bug(f"warehouse.InventoryItem.sku: @Column not in IR (custom field annotations not lowered)")

# ============================================================================
# 12. Services and RPCs
# ============================================================================
print("--- 12. Services/RPCs ---")

fleet_svc = get_service("golden.FleetService")
if fleet_svc:
    # Service-level annotation — KNOWN BUG
    if "ServiceConfig" not in type_annotations(fleet_svc):
        known_bug("FleetService: @ServiceConfig not in IR (service-level custom annotations not lowered)")

    # void input
    r = rpc_by_name(fleet_svc, "ListVehicles")
    if r:
        if not r.get("input", {}).get("isVoid"):
            error("FleetService.ListVehicles: input should be void")

    # void output
    r = rpc_by_name(fleet_svc, "DecommissionVehicle")
    if r:
        if not r.get("output", {}).get("isVoid"):
            error("FleetService.DecommissionVehicle: output should be void")

    # server streaming
    r = rpc_by_name(fleet_svc, "TrackVehicle")
    if r:
        if not r.get("output", {}).get("isStream"):
            error("FleetService.TrackVehicle: output should be stream")

    # RPC-level annotation
    r = rpc_by_name(fleet_svc, "ListVehicles")
    if r:
        rpc_anns = [a["name"] for a in r.get("annotations", [])]
        if "MethodConfig" not in rpc_anns:
            error("FleetService.ListVehicles: missing @MethodConfig on rpc")

wh_svc = get_service("golden.WarehouseService")
if wh_svc:
    # client streaming
    r = rpc_by_name(wh_svc, "BulkIngest")
    if r and not r.get("input", {}).get("isStream"):
        error("WarehouseService.BulkIngest: input should be stream")
    # bidi streaming
    r = rpc_by_name(wh_svc, "SyncInventory")
    if r:
        if not r.get("input", {}).get("isStream"):
            error("WarehouseService.SyncInventory: input should be stream")
        if not r.get("output", {}).get("isStream"):
            error("WarehouseService.SyncInventory: output should be stream")

# Inline request with oneof (our parser fix!)
ship_svc = get_service("golden.ShipmentService")
if ship_svc:
    r = rpc_by_name(ship_svc, "CreateShipment")
    if r:
        inp_msg = r.get("input", {}).get("type", {}).get("messageType", {})
        inline_oneofs = [o["name"] for o in inp_msg.get("oneofs", [])]
        if "delivery_method" not in inline_oneofs:
            error("CreateShipment inline input: missing oneof 'delivery_method'")
        else:
            dm = next(o for o in inp_msg["oneofs"] if o["name"] == "delivery_method")
            dm_fields = [f["name"] for f in dm.get("fields", [])]
            for expected in ["standard", "express", "scheduled"]:
                if expected not in dm_fields:
                    error(f"CreateShipment.delivery_method: missing field '{expected}'")

order_svc = get_service("golden.OrderService")
if order_svc:
    r = rpc_by_name(order_svc, "OrderStatusFeed")
    if r:
        if not r.get("input", {}).get("isStream"):
            error("OrderService.OrderStatusFeed: input should be stream")
        if not r.get("output", {}).get("isStream"):
            error("OrderService.OrderStatusFeed: output should be stream")

# ============================================================================
# 13. Collection types (map, array, optional)
# ============================================================================
print("--- 13. Collection types ---")

meta = get_type("common.Metadata")
if meta:
    if not has_map_type(field_by_name(meta, "labels")):
        error("Metadata.labels: should be map type")
    if not has_map_type(field_by_name(meta, "scores")):
        error("Metadata.scores: should be map type")
    notes = field_by_name(meta, "notes")
    if notes and not notes.get("isOptional"):
        error("Metadata.notes: should be optional")

if vehicle:
    mh = field_by_name(vehicle, "maintenance_history")
    if mh and not mh.get("isRepeated"):
        error("Vehicle.maintenance_history: should be repeated")
    if not has_map_type(field_by_name(vehicle, "telemetry")):
        error("Vehicle.telemetry: should be map type")

# ============================================================================
# 14. Generic type expansion
# ============================================================================
print("--- 14. Generic expansion ---")

lrv = get_type("golden.ListResponseVehicle")
if lrv:
    data = field_by_name(lrv, "data")
    if data is None:
        error("ListResponseVehicle: missing 'data' field")
    else:
        ty = field_type_full(data)
        if ty and "Vehicle" not in str(ty):
            known_bug(f"ListResponseVehicle.data: type should reference Vehicle, got {ty}")

# ============================================================================
# 15. @reserved not in fields
# ============================================================================
print("--- 15. @reserved ---")

if vehicle:
    nums = {f["number"] for f in vehicle.get("fields", [])}
    if 15 in nums or 16 in nums:
        error("Vehicle: @reserved(15)/@reserved(16) leaked as fields")

if order_t:
    nums = {f["number"] for f in order_t.get("fields", [])}
    if 23 in nums or 24 in nums:
        error("Order: @reserved(23)/@reserved(24) leaked as fields")

# ============================================================================
# 16. Inline RPC types exist with correct structure
# ============================================================================
print("--- 16. Inline RPC types ---")

if fleet_svc:
    r = rpc_by_name(fleet_svc, "CreateVehicle")
    if r:
        inp = r.get("input", {}).get("type", {}).get("messageType", {})
        inp_fields = [f["name"] for f in inp.get("fields", [])]
        for expected in ["plate_number", "vehicle_type", "cargo_capacity", "max_payload"]:
            if expected not in inp_fields:
                error(f"CreateVehicle inline input: missing field '{expected}'")

    r = rpc_by_name(fleet_svc, "AssignDriver")
    if r:
        out = r.get("output", {}).get("type", {}).get("messageType", {})
        out_fields = [f["name"] for f in out.get("fields", [])]
        for expected in ["assignment", "was_reassigned"]:
            if expected not in out_fields:
                error(f"AssignDriver inline output: missing field '{expected}'")

# ============================================================================
# Summary
# ============================================================================
print()
print("=" * 60)

if errors:
    print(f"FAILED: {len(errors)} error(s)")
    for e in errors:
        print(f"  ERROR:     {e}")
else:
    print("ALL CHECKS PASSED")

if known_bugs:
    print(f"\nKNOWN BUGS: {len(known_bugs)}")
    for b in known_bugs:
        print(f"  KNOWN BUG: {b}")

sys.exit(1 if errors else 0)
