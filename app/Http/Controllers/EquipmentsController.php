<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Http\Controllers\Controller;
use App\Models\Equipments;
use Illuminate\View\View;
use Illuminate\Http\RedirectResponse;

class EquipmentsController extends Controller
{
    public function index(LogsController $logsController): View|RedirectResponse
    {
        try {
            $equipments = Equipments::all();
            $logsController->createLog(__METHOD__, 'success', 'User is viewing Equipments list', null, null);
            return view('equipments.index', compact('equipments'));
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to list Equipments: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to load data: ' . $e->getMessage());
        }
    }

    public function create(LogsController $logsController): View|RedirectResponse
    {
        try {
            $logsController->createLog(__METHOD__, 'success', 'User is opening Equipment create form', null, null);
            return view('equipments.create');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to open Equipment create form: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to open create form: ' . $e->getMessage());
        }
    }

    public function store(Request $request, LogsController $logsController): RedirectResponse
    {
        try {
            $request->validate([
                'name' => 'required'
            ]);

            Equipments::create([
                'name' => $request->name,
            ]);

            $logsController->createLog(__METHOD__, 'success', 'User created a new Equipment', null, null);
            return redirect()->route('equipments.index')->with('success', 'Equipment created successfully');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to create Equipment: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to create Equipment: ' . $e->getMessage());
        }
    }

    public function show($id, LogsController $logsController): View|RedirectResponse
    {
        dd('This method is not implemented yet.'); // Placeholder for future implementation
    }

    public function edit($id, LogsController $logsController): View|RedirectResponse
    {
        try {
            $equipment = Equipments::findOrFail($id);
            $logsController->createLog(__METHOD__, 'success', 'User opened edit form for Equipment (ID: ' . $id . ')', null, null);
            return view('equipments.edit', compact('equipment'));
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to open edit form for Equipment (ID: ' . $id . '): ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to open edit form: ' . $e->getMessage());
        }
    }

    public function update(Request $request, $id, LogsController $logsController): RedirectResponse
    {
        try {
            $request->validate([
                'name' => 'required'
            ]);

            $equipment = Equipments::findOrFail($id);
            $equipment->update([
                'name' => $request->name,
            ]);

            $logsController->createLog(__METHOD__, 'success', 'User updated Equipment (ID: ' . $id . ')', null, null);
            return redirect()->route('equipments.index')->with('success', 'Equipment updated successfully');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to update Equipment (ID: ' . $id . '): ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to update Equipment: ' . $e->getMessage());
        }
    }

    public function destroy($id, LogsController $logsController): RedirectResponse
    {
        try {
            $equipment = Equipments::findOrFail($id);
            $equipment->delete();

            $logsController->createLog(__METHOD__, 'success', 'User deleted Equipment (ID: ' . $id . ')', null, null);
            return redirect()->route('equipments.index')->with('success', 'Equipment deleted successfully');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to delete Equipment (ID: ' . $id . '): ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to delete Equipment: ' . $e->getMessage());
        }
    }
}
