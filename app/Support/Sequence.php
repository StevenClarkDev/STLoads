<?php

namespace App\Support;

use Illuminate\Support\Facades\DB;

class Sequence
{
    public static function next(string $key): int
    {
        return DB::transaction(function () use ($key) {
            $row = DB::table('sequences')->where('key', $key)->lockForUpdate()->first();
            if (!$row) {
                DB::table('sequences')->insert(['key' => $key, 'value' => 1]);
                return 1;
            }
            $next = $row->value + 1;
            DB::table('sequences')->where('key', $key)->update(['value' => $next]);
            return $next;
        }, 3);
    }
}
