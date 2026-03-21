<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('users', function (Blueprint $table) {
            $table->string('nationality')->nullable()->after('address');
            $table->string('company_name')->nullable()->after('nationality');
            $table->string('registration_number')->nullable()->after('company_name');
            $table->string('tax_id')->nullable()->after('registration_number');
            $table->string('country_of_incorporation')->nullable()->after('tax_id');
            $table->boolean('consent_sanctions_screening')->default(false)->after('country_of_incorporation');
            $table->boolean('politically_exposed_person')->default(false)->after('consent_sanctions_screening');
            $table->text('source_of_funds')->nullable()->after('politically_exposed_person');
            $table->boolean('agree_aml_policies')->default(false)->after('source_of_funds');
        });
    }

    public function down(): void
    {
        Schema::table('users', function (Blueprint $table) {
            $table->dropColumn([
                'nationality',
                'company_name',
                'registration_number',
                'tax_id',
                'country_of_incorporation',
                'consent_sanctions_screening',
                'politically_exposed_person',
                'source_of_funds',
                'agree_aml_policies',
            ]);
        });
    }
};
