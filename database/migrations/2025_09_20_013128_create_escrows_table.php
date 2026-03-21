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
        Schema::create('escrows', function (Blueprint $t) {
            $t->id();
            $t->uuid('leg_id')->index();               // your leg PK/UUID
            $t->foreignId('payer_user_id')->constrained('users');   // load creator
            $t->foreignId('payee_user_id')->constrained('users');   // carrier
            $t->string('currency', 3);
            $t->bigInteger('amount');                  // minor units
            $t->bigInteger('platform_fee')->default(0);
            $t->string('status')->default('unfunded'); // unfunded|funded|released|refunded|on_hold|failed
            $t->string('transfer_group')->nullable();
            $t->string('payment_intent_id')->nullable();
            $t->string('charge_id')->nullable();
            $t->string('transfer_id')->nullable();
            $t->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('escrows');
    }
};
