<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Http\Controllers\Controller;
use App\Models\CommodityTypes;
use Illuminate\View\View;
use Illuminate\Http\RedirectResponse;

class CommodityTypesController extends Controller
{
    public function index(LogsController $logsController): View|RedirectResponse
    {
        try {
            $commodity_types = CommodityTypes::all();
            $logsController->createLog(__METHOD__, 'success', 'User is viewing Commodity Types list', null, null);
            return view('commodity_types.index', compact('commodity_types'));
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to list Commodity Types: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to load data: ' . $e->getMessage());
        }
    }

    public function create(LogsController $logsController): View|RedirectResponse
    {
        try {
            $logsController->createLog(__METHOD__, 'success', 'User is opening Commodity Type create form', null, null);
            return view('commodity_types.create');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to open Commodity Type create form: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to open create form: ' . $e->getMessage());
        }
    }

    public function store(Request $request, LogsController $logsController): RedirectResponse
    {
        try {
            $request->validate([
                'name' => 'required'
            ]);

            CommodityTypes::create([
                'name' => $request->name,
            ]);

            $logsController->createLog(__METHOD__, 'success', 'User created a new Commodity Type', null, null);
            return redirect()->route('commodity_types.index')->with('success', 'Commodity Type created successfully');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to create Commodity Type: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to create Commodity Type: ' . $e->getMessage());
        }
    }

    public function show($id, LogsController $logsController): View|RedirectResponse
    {
        dd('This method is not implemented yet.'); // Placeholder for future implementation
    }

    public function edit($id, LogsController $logsController): View|RedirectResponse
    {
        try {
            $commodity_type = CommodityTypes::findOrFail($id);
            $logsController->createLog(__METHOD__, 'success', 'User opened edit form for Commodity Type (ID: ' . $id . ')', null, null);
            return view('commodity_types.edit', compact('commodity_type'));
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to open edit form for Commodity Type (ID: ' . $id . '): ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to open edit form: ' . $e->getMessage());
        }
    }

    public function update(Request $request, $id, LogsController $logsController): RedirectResponse
    {
        try {
            $request->validate([
                'name' => 'required'
            ]);

            $commodity_type = CommodityTypes::findOrFail($id);
            $commodity_type->update([
                'name' => $request->name,
            ]);

            $logsController->createLog(__METHOD__, 'success', 'User updated Commodity Type (ID: ' . $id . ')', null, null);
            return redirect()->route('commodity_types.index')->with('success', 'Commodity Type updated successfully');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to update Commodity Type (ID: ' . $id . '): ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to update Commodity Type: ' . $e->getMessage());
        }
    }

    public function destroy($id, LogsController $logsController): RedirectResponse
    {
        try {
            $commodity_type = CommodityTypes::findOrFail($id);
            $commodity_type->delete();

            $logsController->createLog(__METHOD__, 'success', 'User deleted Commodity Type (ID: ' . $id . ')', null, null);
            return redirect()->route('commodity_types.index')->with('success', 'Commodity Type deleted successfully');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to delete Commodity Type (ID: ' . $id . '): ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to delete Commodity Type: ' . $e->getMessage());
        }
    }
}
