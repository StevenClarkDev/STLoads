<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Models\User;
use App\Models\Equipments;
use App\Models\CommodityTypes;
use App\Models\Locations;
use App\Models\LoadType;

class LoadController extends Controller
{
    public function index(LogsController $logsController)
    {
        try {
            $logsController->createLog(__METHOD__, 'success', 'User is attempting to index load in', null, null);
            return view('load.index');
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'An error occurred while processing your request.'. $e->getMessage());
        }
    }

    public function add(LogsController $logsController)
    {
        try {
            $load_types = LoadType::all();
            $equipments = Equipments::all();
            $commodity_types = CommodityTypes::all();
            $locations = Locations::all();
            $logsController->createLog(__METHOD__, 'success', 'User is attempting to add load in', null, null);
            return view('load.add', compact('load_types', 'equipments', 'commodity_types', 'locations'));
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'An error occurred while processing your request.'. $e->getMessage());
        }
    }

}
