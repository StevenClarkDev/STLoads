<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('stloads_handoffs', function (Blueprint $table) {
            $table->id();

            // ── Identity ──────────────────────────────────────────────
            $table->string('tms_load_id', 100)->index();          // internal load ID from TMS
            $table->string('tenant_id', 100)->nullable()->index(); // TMS tenant/account
            $table->string('external_handoff_id', 100)->nullable()->unique(); // unique handoff ref

            // ── Link to local load once published ─────────────────────
            $table->foreignId('load_id')->nullable()->constrained('loads')->nullOnDelete();

            // ── Handoff lifecycle ─────────────────────────────────────
            $table->string('status', 30)->default('queued');
            // queued | push_in_progress | published | push_failed | requeue_required | withdrawn | closed

            // ── Freight classification ────────────────────────────────
            $table->string('party_type', 50)->nullable();         // shipper, broker, forwarder
            $table->string('freight_mode', 50)->nullable();       // FTL, LTL, Reefer, etc.
            $table->string('equipment_type', 100)->nullable();
            $table->string('commodity_description', 255)->nullable();
            $table->decimal('weight', 12, 2)->nullable();
            $table->string('weight_unit', 10)->default('lbs');
            $table->integer('piece_count')->nullable();
            $table->json('temperature_data')->nullable();         // reefer min/max/unit
            $table->json('container_data')->nullable();           // drayage container/port
            $table->json('securement_data')->nullable();          // flatbed tarp/securement
            $table->boolean('is_hazardous')->default(false);

            // ── Route & timing ────────────────────────────────────────
            $table->string('pickup_city', 100)->nullable();
            $table->string('pickup_state', 50)->nullable();
            $table->string('pickup_zip', 20)->nullable();
            $table->string('pickup_country', 5)->default('US');
            $table->text('pickup_address')->nullable();
            $table->dateTime('pickup_window_start')->nullable();
            $table->dateTime('pickup_window_end')->nullable();
            $table->text('pickup_instructions')->nullable();
            $table->string('pickup_appointment_ref', 100)->nullable();

            $table->string('dropoff_city', 100)->nullable();
            $table->string('dropoff_state', 50)->nullable();
            $table->string('dropoff_zip', 20)->nullable();
            $table->string('dropoff_country', 5)->default('US');
            $table->text('dropoff_address')->nullable();
            $table->dateTime('dropoff_window_start')->nullable();
            $table->dateTime('dropoff_window_end')->nullable();
            $table->text('dropoff_instructions')->nullable();
            $table->string('dropoff_appointment_ref', 100)->nullable();

            // ── Commercial projection ─────────────────────────────────
            $table->decimal('board_rate', 12, 2)->nullable();
            $table->string('rate_currency', 5)->default('USD');
            $table->json('accessorial_flags')->nullable();
            $table->string('bid_type', 20)->default('open');      // open | fixed
            $table->string('quote_status', 30)->nullable();
            $table->string('tender_posture', 30)->nullable();

            // ── Compliance & readiness ─────────────────────────────────
            $table->boolean('compliance_passed')->default(false);
            $table->json('compliance_summary')->nullable();       // blocking flags, doc status
            $table->json('required_documents_status')->nullable();
            $table->string('readiness', 30)->default('pending');  // pending | ready | blocked

            // ── Handoff metadata ──────────────────────────────────────
            $table->string('pushed_by', 100)->nullable();
            $table->string('push_reason', 255)->nullable();
            $table->string('source_module', 50)->nullable();      // dashboard, quote-desk, tender-desk, etc.
            $table->dateTime('queued_at')->nullable();
            $table->dateTime('published_at')->nullable();
            $table->dateTime('withdrawn_at')->nullable();
            $table->dateTime('closed_at')->nullable();
            $table->integer('retry_count')->default(0);
            $table->text('last_push_result')->nullable();
            $table->string('payload_version', 20)->default('1.0');

            // ── The full raw payload from TMS ─────────────────────────
            $table->json('raw_payload')->nullable();

            $table->timestamps();

            $table->index(['status', 'created_at']);
            $table->index(['tms_load_id', 'status']);
        });

        Schema::create('stloads_handoff_events', function (Blueprint $table) {
            $table->id();
            $table->foreignId('handoff_id')->constrained('stloads_handoffs')->cascadeOnDelete();
            $table->string('event_type', 50);
            // push_requested | push_accepted | push_succeeded | push_failed
            // requeue_requested | withdrawn | closed | reconciled
            $table->string('performed_by', 100)->nullable();
            $table->string('source_module', 50)->nullable();
            $table->text('payload_snapshot')->nullable();
            $table->text('result')->nullable();
            $table->timestamps();

            $table->index(['handoff_id', 'event_type']);
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('stloads_handoff_events');
        Schema::dropIfExists('stloads_handoffs');
    }
};
