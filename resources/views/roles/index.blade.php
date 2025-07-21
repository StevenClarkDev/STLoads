@extends('layout.app')
@section('content')
    <!-- Main Content Column -->
    <div class="col-xl-9 box-col-6 p-3">
        <div class="card h-100">
            <div class="card-body p-0">
                <div class="card">
                    <div class="card-header pb-0 card-no-border">
                        <h4>Roles List</h4>
                        <span>See roles below.</span>
                    </div>
                    <div class="card-body">
                        <div class="table-responsive">
                            <table class="table table-striped w-100" id="user-approval-table">
                                <thead>
                                    <tr>
                                        <th>S No.</th>
                                        <th>Name</th>
                                        <th>Action</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    @foreach ($roles as $key => $role)
                                        <tr>
                                            <td>{{ ++$i }}</td>
                                            <td>{{ $role->name }}</td>
                                            <td>
                                                <a class="btn btn-info btn-sm" href="{{ route('roles.show', $role->id) }}"><i
                                                        class="fa fa-list"></i> Show</a>
                                                @can('role-edit')
                                                    <a class="btn btn-primary btn-sm"
                                                        href="{{ route('roles.edit', $role->id) }}"><i
                                                            class="fa fa-pencil"></i> Edit</a>
                                                @endcan

                                                {{-- @can('role-delete')
                                                    <form method="POST" action="{{ route('roles.destroy', $role->id) }}"
                                                        style="display:inline">
                                                        @csrf
                                                        @method('DELETE')

                                                        <button type="submit" class="btn btn-danger btn-sm"><i
                                                                class="fa-solid fa-trash"></i> Delete</button>
                                                    </form>
                                                @endcan --}}
                                            </td>
                                        </tr>
                                    @endforeach
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
@endsection
