<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void {
        Schema::table('loads', function (Blueprint $table) {
            $table->string('load_number', 50)->nullable()->unique()->after('id');
            $table->unsignedInteger('leg_count')->default(0)->after('load_number');
        });

        Schema::table('load_legs', function (Blueprint $table) {
            $table->unsignedInteger('leg_no')->default(1)->after('id');
            $table->string('leg_code', 70)->nullable()->after('leg_no'); // e.g., FF-202508-0001-L02
            $table->index(['load_id', 'leg_no']);
            $table->unique(['load_id', 'leg_no']); // one leg number per load
        });
    }

    public function down(): void {
        Schema::table('load_legs', function (Blueprint $table) {
            $table->dropUnique(['load_id', 'leg_no']);
            $table->dropIndex(['load_id', 'leg_no']);
            $table->dropColumn(['leg_no','leg_code']);
        });
        Schema::table('loads', function (Blueprint $table) {
            $table->dropUnique(['load_number']);
            $table->dropColumn(['load_number','leg_count']);
        });
    }
};
