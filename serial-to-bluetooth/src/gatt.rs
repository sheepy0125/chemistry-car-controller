/*!
 * GATT UUIDs
 * Created by sheepy0125 | MIT License | 2023-02-19
 */

/// Service UUID for our GATT
pub const SERVICE_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x01ff0100ba5ef4ee5ca1eb1e5e4b1ce0);

/// Characteristic UUID for the TX
/// Write-only - size: 1
pub const TX_CHARACTERISTIC_SIZE: usize = 1_usize;
pub const TX_CHARACTERISTIC_UUID: uuid::Uuid =
    uuid::Uuid::from_u128(0x01ff0101ba5ef4ee5ca1eb1e5e4b1ce0);

/// Characteristic UUID for the RX
/// Read-only - size: 244
pub const RX_CHARACTERISTIC_SIZE: usize = 244_usize;
pub const RX_CHARACTERISTIC_UUID: uuid::Uuid =
    uuid::Uuid::from_u128(0x01ff0101ba5ef4ee5ca1eb1e5e4b1ce1);
