<?php

namespace Database\Seeders;

use App\Models\User;
use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\Hash;
use Spatie\Permission\Models\Role;

class AdminSeeder extends Seeder
{
    public function run(): void
    {
        // Ensure the Admin role exists
        $role = Role::firstOrCreate(
            ['name' => 'Admin', 'guard_name' => 'web']
        );

        // Create or update the admin user
        $admin = User::updateOrCreate(
            ['email' => 'admin@stloads.com'],
            [
                'name'              => 'Admin',
                'password'          => Hash::make('admin123'),
                'email_verified_at' => now(),
                'status'            => 1,
            ]
        );

        // Assign the Admin role (safe to call even if already assigned)
        if (!$admin->hasRole('Admin')) {
            $admin->assignRole('Admin');
        }

        $this->command->info("Admin user ready — email: admin@stloads.com");
    }
}
