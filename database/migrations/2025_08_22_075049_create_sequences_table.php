<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void {
        Schema::create('sequences', function (Blueprint $table) {
            $table->string('key', 100)->primary();   // e.g., "FF-202508"
            $table->unsignedBigInteger('value')->default(0);
            $table->timestamps();
        });
    }
    public function down(): void {
        Schema::dropIfExists('sequences');
    }
};
