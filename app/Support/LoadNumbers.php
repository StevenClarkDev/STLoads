<?php
// app/Support/LoadNumbers.php
namespace App\Support;

use Carbon\Carbon;

class LoadNumbers
{
    public static function prefixForRole(int $roleId): string
    {
        // adjust to your real role IDs
        return match($roleId) {
            5 => 'FF',           // Freight Forwarder
            4 => 'BRK',          // Broker (example)
            2 => 'SHP',          // Shipper (example)
            default => 'LD',     // Fallback
        };
    }

    public static function generateLoadNumber(int $roleId): string
    {
        $prefix = self::prefixForRole($roleId);
        $period = now()->format('Ym'); // 202508
        $seqKey = "{$prefix}-{$period}";
        $n = Sequence::next($seqKey);  // 1,2,3...
        return sprintf('%s-%s-%04d', $prefix, $period, $n); // FF-202508-0001
    }

    public static function legCode(string $loadNumber, int $legNo): string
    {
        return sprintf('%s-L%02d', $loadNumber, $legNo); // FF-202508-0001-L02
    }
}
