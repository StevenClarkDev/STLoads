<?php

namespace Database\Seeders;

use Illuminate\Database\Seeder;
use App\Models\LoadType;

class LoadTypesSeeder extends Seeder
{
    /**
     * Run the database seeds.
     */
    public function run(): void
    {
        $loadTypes = [
            'FTL (Full Truckload)',
            'LTL (Less Than Truckload)',
            'Reefer',
            'Oversized / Overdimensional',
            'Hazmat',
            'Flatbed',
            'Dry Van',
            'Tanker',
            'Intermodal',
            'Expedited',
            'Partial Truckload',
            'Heavy Haul'
        ];

        foreach ($loadTypes as $type) {
            LoadType::firstOrCreate(['name' => $type]);
        }
    }
}
