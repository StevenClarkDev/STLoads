<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        // ── External references from TMS side ─────────────────
        Schema::create('stloads_external_refs', function (Blueprint $table) {
            $table->id();
            $table->foreignId('handoff_id')->constrained('stloads_handoffs')->cascadeOnDelete();
            $table->string('ref_type', 50);          // e.g. 'tms_shipment', 'tms_dispatch', 'tms_invoice', 'customer_po', 'bol'
            $table->string('ref_value', 255);         // the actual reference ID / number
            $table->string('ref_source', 100)->nullable(); // which TMS module produced this ref
            $table->timestamps();

            $table->index(['handoff_id', 'ref_type']);
            $table->index('ref_value');
        });

        // ── Sync errors / reconciliation warnings ─────────────
        Schema::create('stloads_sync_errors', function (Blueprint $table) {
            $table->id();
            $table->foreignId('handoff_id')->nullable()->constrained('stloads_handoffs')->nullOnDelete();
            $table->string('error_class', 80);        // e.g. 'push_failed', 'duplicate_publish', 'compliance_blocked', 'delivered_still_open', 'withdraw_mismatch'
            $table->string('severity', 20)->default('warning'); // 'info', 'warning', 'error', 'critical'
            $table->string('title', 255);
            $table->text('detail')->nullable();
            $table->string('source_module', 100)->nullable();
            $table->string('performed_by', 255)->nullable();
            $table->boolean('resolved')->default(false);
            $table->timestamp('resolved_at')->nullable();
            $table->string('resolved_by', 255)->nullable();
            $table->text('resolution_note')->nullable();
            $table->timestamps();

            $table->index(['error_class', 'resolved']);
            $table->index(['severity', 'resolved']);
            $table->index('handoff_id');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('stloads_sync_errors');
        Schema::dropIfExists('stloads_external_refs');
    }
};
