<?php

namespace App\Support;

final class Roles
{
    public const ADMIN = 'Admin';
    public const SHIPPER = 'Shipper';
    public const CARRIER = 'Carrier';
    public const BROKER = 'Broker';
    public const FREIGHT_FORWARDER = 'Freight Forwarder';

    // Users who can own loads
    public const LOAD_OWNERS = [
        self::SHIPPER,
        self::BROKER,
        self::FREIGHT_FORWARDER,
    ];
}
