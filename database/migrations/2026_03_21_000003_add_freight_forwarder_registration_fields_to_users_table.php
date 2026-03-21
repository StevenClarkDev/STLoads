<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('users', function (Blueprint $table) {
            $table->string('trade_name')->nullable()->after('company_name');
            $table->date('incorporation_date')->nullable()->after('trade_name');
            $table->string('director_name')->nullable()->after('incorporation_date');
            $table->date('director_dob')->nullable()->after('director_name');
            $table->string('ubo_name')->nullable()->after('director_dob');
            $table->date('ubo_dob')->nullable()->after('ubo_name');
            $table->string('ubo_nationality')->nullable()->after('ubo_dob');
            $table->text('ubo_address')->nullable()->after('ubo_nationality');
            $table->string('fmc_license')->nullable()->after('ubo_address');
            $table->string('nvocc_reg')->nullable()->after('fmc_license');
            $table->string('surety_bond')->nullable()->after('nvocc_reg');
            $table->string('customs_broker_license')->nullable()->after('surety_bond');
            $table->string('iata_accreditation')->nullable()->after('customs_broker_license');
            $table->string('eori_number')->nullable()->after('iata_accreditation');
            $table->string('secp_reg')->nullable()->after('eori_number');
            $table->string('chamber_reg')->nullable()->after('secp_reg');
            $table->string('policy_number')->nullable()->after('chamber_reg');
            $table->string('insurer_contact')->nullable()->after('policy_number');
            $table->string('transport_modes')->nullable()->after('insurer_contact');
            $table->string('countries_served')->nullable()->after('transport_modes');
            $table->string('customs_brokerage')->nullable()->after('countries_served');
            $table->string('consolidation_services')->nullable()->after('customs_brokerage');
            $table->string('warehousing')->nullable()->after('consolidation_services');
            $table->unsignedSmallInteger('years_in_operation')->nullable()->after('warehousing');
            $table->string('annual_volume')->nullable()->after('years_in_operation');
            $table->string('monthly_transaction_volume')->nullable()->after('annual_volume');
            $table->boolean('ofac_consent')->default(false)->after('monthly_transaction_volume');
        });
    }

    public function down(): void
    {
        Schema::table('users', function (Blueprint $table) {
            $table->dropColumn([
                'trade_name',
                'incorporation_date',
                'director_name',
                'director_dob',
                'ubo_name',
                'ubo_dob',
                'ubo_nationality',
                'ubo_address',
                'fmc_license',
                'nvocc_reg',
                'surety_bond',
                'customs_broker_license',
                'iata_accreditation',
                'eori_number',
                'secp_reg',
                'chamber_reg',
                'policy_number',
                'insurer_contact',
                'transport_modes',
                'countries_served',
                'customs_brokerage',
                'consolidation_services',
                'warehousing',
                'years_in_operation',
                'annual_volume',
                'monthly_transaction_volume',
                'ofac_consent',
            ]);
        });
    }
};
