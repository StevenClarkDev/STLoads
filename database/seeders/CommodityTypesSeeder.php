<?php

namespace Database\Seeders;

use Illuminate\Database\Seeder;
use App\Models\CommodityType;
use App\Models\CommodityTypes;

class CommodityTypesSeeder extends Seeder
{
    /**
     * Run the database seeds.
     */
    public function run(): void
    {
        $commodityTypes = [
            'Electronics',
            'Machinery',
            'Food & Beverages',
            'Automotive Parts',
            'Building Materials',
            'Chemicals',
            'Textiles',
            'Pharmaceuticals',
            'Household Goods',
            'Furniture',
            'Paper Products',
            'Metals'
        ];

        foreach ($commodityTypes as $type) {
            CommodityTypes::firstOrCreate(['name' => $type]);
        }
    }
}
