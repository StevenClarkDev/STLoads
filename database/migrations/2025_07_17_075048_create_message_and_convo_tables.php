<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        // conversations
        Schema::create('conversations', function (Blueprint $t) {
            $t->id();
            $t->foreignId('load_id')->constrained();
            $t->foreignId('shipper_id')->constrained('users'); // load owner (shipper/broker/forwarder)
            $t->foreignId('carrier_id')->constrained('users'); // carrier who bids/initiates
            $t->timestamps();
            $t->unique(['load_id', 'carrier_id']); // <= single thread per load x carrier
        });

        // messages (unchanged)
        Schema::create('messages', function (Blueprint $t) {
            $t->id();
            $t->foreignId('conversation_id')->constrained();
            $t->foreignId('user_id')->constrained();
            $t->text('body')->nullable();
            $t->json('meta')->nullable(); // store bid amount etc
            $t->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('conversations');
        Schema::dropIfExists('messages');
    }
};
