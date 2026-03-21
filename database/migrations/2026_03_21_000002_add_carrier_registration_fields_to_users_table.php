<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('users', function (Blueprint $table) {
            $table->string('gov_id_number')->nullable()->after('agree_aml_policies');
            $table->string('cdl_number')->nullable()->after('gov_id_number');
            $table->date('cdl_expiry')->nullable()->after('cdl_number');
            $table->string('cdl_class')->nullable()->after('cdl_expiry');
            $table->string('regulatory_country')->nullable()->after('cdl_class');
            $table->string('usdot_number')->nullable()->after('regulatory_country');
            $table->string('mc_number')->nullable()->after('usdot_number');
            $table->string('ntn')->nullable()->after('mc_number');
            $table->string('vat_number')->nullable()->after('ntn');
            $table->date('insurance_expiry')->nullable()->after('vat_number');
            $table->string('coverage_limits')->nullable()->after('insurance_expiry');
            $table->string('insurer_name')->nullable()->after('coverage_limits');
            $table->string('vehicle_reg')->nullable()->after('insurer_name');
            $table->string('vehicle_make_model')->nullable()->after('vehicle_reg');
            $table->smallInteger('vehicle_year')->nullable()->after('vehicle_make_model');
            $table->string('vehicle_type')->nullable()->after('vehicle_year');
            $table->string('load_capacity')->nullable()->after('vehicle_type');
            $table->text('company_address')->nullable()->after('load_capacity');
            $table->string('bank_account')->nullable()->after('company_address');
            $table->boolean('criminal_declaration')->default(false)->after('bank_account');
            $table->boolean('terms_agreed')->default(false)->after('criminal_declaration');
        });
    }

    public function down(): void
    {
        Schema::table('users', function (Blueprint $table) {
            $table->dropColumn([
                'gov_id_number', 'cdl_number', 'cdl_expiry', 'cdl_class',
                'regulatory_country', 'usdot_number', 'mc_number', 'ntn', 'vat_number',
                'insurance_expiry', 'coverage_limits', 'insurer_name',
                'vehicle_reg', 'vehicle_make_model', 'vehicle_year', 'vehicle_type', 'load_capacity',
                'company_address', 'bank_account', 'criminal_declaration', 'terms_agreed',
            ]);
        });
    }
};
