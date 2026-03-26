<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('users', function (Blueprint $table) {
            $table->string('ucr_hcc_no')->nullable()->after('ssn_no');
            $table->string('mc_cbsa_usdot_no')->nullable()->after('ucr_hcc_no');
        });
    }

    public function down(): void
    {
        Schema::table('users', function (Blueprint $table) {
            $table->dropColumn(['ucr_hcc_no', 'mc_cbsa_usdot_no']);
        });
    }
};
