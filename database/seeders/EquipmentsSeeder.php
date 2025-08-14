<?php

namespace Database\Seeders;

use Illuminate\Database\Seeder;
use App\Models\Equipment;
use App\Models\Equipments;

class EquipmentsSeeder extends Seeder
{
    /**
     * Run the database seeds.
     */
    public function run(): void
    {
        $equipments = [
            'Dry Van',
            'Reefer',
            'Flatbed',
            'Step Deck',
            'Lowboy',
            'Tanker',
            'Conestoga',
            'Power Only',
            'Box Truck',
            'Hot Shot',
            'Car Hauler',
            'Container Chassis'
        ];

        foreach ($equipments as $equipment) {
            Equipments::firstOrCreate(['name' => $equipment]);
        }
    }
}
