<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        // ── Add TMS status tracking to handoffs ───────────────
        Schema::table('stloads_handoffs', function (Blueprint $table) {
            $table->string('tms_status', 50)->nullable()->after('status');
            // The last known lifecycle status from the TMS dispatch side:
            // dispatched | in_transit | at_pickup | at_delivery | delivered | cancelled | invoiced | settled

            $table->dateTime('tms_status_at')->nullable()->after('tms_status');
            // When the TMS last reported this status

            $table->dateTime('last_webhook_at')->nullable()->after('payload_version');
            // Timestamp of the most recent webhook received for this handoff

            $table->index(['tms_status', 'status']);
        });

        // ── Reconciliation log — every reconcile action ───────
        Schema::create('stloads_reconciliation_log', function (Blueprint $table) {
            $table->id();
            $table->foreignId('handoff_id')->constrained('stloads_handoffs')->cascadeOnDelete();
            $table->string('action', 50);
            // status_update | auto_withdraw | auto_close | auto_archive | rate_update | mismatch_detected | force_sync

            $table->string('tms_status_from', 50)->nullable();
            $table->string('tms_status_to', 50)->nullable();
            $table->string('stloads_status_from', 50)->nullable();
            $table->string('stloads_status_to', 50)->nullable();

            $table->text('detail')->nullable();
            $table->string('triggered_by', 100)->nullable();  // 'webhook', 'cron', 'manual', user email
            $table->json('webhook_payload')->nullable();       // raw webhook body for audit

            $table->timestamps();

            $table->index(['handoff_id', 'action']);
            $table->index('created_at');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('stloads_reconciliation_log');

        Schema::table('stloads_handoffs', function (Blueprint $table) {
            $table->dropIndex(['tms_status', 'status']);
            $table->dropColumn(['tms_status', 'tms_status_at', 'last_webhook_at']);
        });
    }
};
