<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration {
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::table('users', function (Blueprint $t) {
            $t->string('stripe_connect_account_id')->nullable();
            $t->boolean('payouts_enabled')->default(false);
            $t->string('kyc_status')->nullable(); // 'pending'|'verified'|'restricted'
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('users', function (Blueprint $table) {
            $table->dropColumn([
                'stripe_connect_account_id',
                'payouts_enabled',
                'kyc_status'
            ]);
        });
    }

};
