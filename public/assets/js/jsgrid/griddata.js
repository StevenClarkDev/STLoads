'use strict';
(function() {
    var db = {
        loadData: function(filter) {
            return $.grep(this.clients, function(client) {
                return (!filter.Name || client.Name.indexOf(filter.Name) > -1)
                    && (!filter.Age || client.Age === filter.Age)
                    && (!filter.Address || client.Address.indexOf(filter.Address) > -1)
                    && (!filter.Country || client.Country === filter.Country)
                    && (filter.Married === undefined || client.Married === filter.Married);
            });
        },
        insertItem: function(insertingClient) {
            this.clients.push(insertingClient);
        },
        updateItem: function(updatingClient) { },

        deleteItem: function(deletingClient) {
            var clientIndex = $.inArray(deletingClient, this.clients);
            this.clients.splice(clientIndex, 1);
        }
    };
    window.db = db;
    db.countries = [
        { Name: "India", Id: 0 },
        { Name: "United States", Id: 1 },
        { Name: "Canada", Id: 2 },
        { Name: "United Kingdom", Id: 3 },
        { Name: "France", Id: 4 },
        { Name: "Brazil", Id: 5 },
        { Name: "China", Id: 6 },
        { Name: "Russia", Id: 7 }
    ];
    db.clients = [
        {
            "ID": "L1",
            "Pickup Location": "Sydney, AU",
            "Delivery Location": "Arizona, US",
            "Pickup Date": "26/09/2022",
            "Delivery Date": "26/09/2022",
            "Bid Status": "<span class=\"badge badge-light-success\">Biddable</span>", 
            "Price": "$2315.00",
            "Progress": "100%",

            "Id": "1",
            "Product": "Samsung S22 ultra ",
            "Order Id": "#F8ST59L",
            "Quantity": "2",
            "Shipped": "<span class=\"badge badge-light-danger\">Out For Delivery</span>",
            "Total": "$25364",

            "Employee Name": "Virat Kohli",
            "Salary": "$12,000",
            "Skill": "<div class=\"progress-showcase\"><div class=\"progress sm-progress-bar\"><div class=\"progress-bar bg-danger\" role=\"progressbar\" style=\"width: 40%\" aria-valuenow=\"50\" aria-valuemin=\"0\" aria-valuemax=\"100\"></div></div></div>",
            "Office": "India",
            "Hours": "4:30",
            "Experience": "12 Year",
        },
        {
            "ID": "L2",
            "Pickup Location": "Sydney, AU",
            "Delivery Location": "Arizona, US",
            "Pickup Date": "26/09/2022",
            "Delivery Date": "26/09/2022",
            "Bid Status": "<span class=\"badge badge-light-danger\">Fixed</span>",
            "Price": "$1598.50",
            "Progress": "60%",

            "Id": "2",
            "Product": "Airpord Pro",
            "Order Id": "#TD6Y56W",
            "Quantity": "1",
            "Shipped": "<span class=\"badge badge-light-danger\">Order Cancelled</span>",
            "Total": "$12457",

            "Employee Name": "Ronaldo",
            "Salary": "$80,000",
            "Skill": "<div class=\"progress-showcase\"><div class=\"progress sm-progress-bar\"><div class=\"progress-bar bg-primary\" role=\"progressbar\" style=\"width: 78%\" aria-valuenow=\"50\" aria-valuemin=\"0\" aria-valuemax=\"100\"></div></div></div>",
            "Office": "UK",
            "Hours": "3:00",
            "Experience": "20 Year",
        },
        {
            "ID": "L3",
            "Pickup Location": "Sydney, AU",
            "Delivery Location": "Arizona, US",
            "Pickup Date": "26/09/2022",
            "Delivery Date": "12/07/2022",
            "Bid Status": "<span class=\"badge badge-light-danger\">Fixed</span>",
            "Price": "$1598.50",
            "Progress": "42%",

            "Id": "3",
            "Product": "OnePlus Nord CE 2 Lite",
            "Order Id": "#9E75CF4",
            "Quantity": "4",
            "Shipped": "<span class=\"badge badge-light-success\">Delivery Completed</span>",
            "Total": "$89241",

            "Employee Name": "Jos Buttler",
            "Salary": "$70,000",
            "Skill": "<div class=\"progress-showcase\"><div class=\"progress sm-progress-bar\"><div class=\"progress-bar bg-success\" role=\"progressbar\" style=\"width: 80%\" aria-valuenow=\"50\" aria-valuemin=\"0\" aria-valuemax=\"100\"></div></div></div>",
            "Office": "Englend",
            "Hours": "4:00",
            "Experience": "8 Year",
        },
        {
            "ID": "L4",
            "Pickup Location": "Sydney, AU",
            "Delivery Location": "Arizona, US",
            "Pickup Date": "26/09/2022",
            "Delivery Date": "26/09/2022",
            "Bid Status": "<span class=\"badge badge-light-success\">Biddable</span>",
            "Price": "$2315.00",
            "Progress": "90%",

            "Id": "4",
            "Product": "i phone 14 pro Max",
            "Order Id": "#1A84RD3",
            "Quantity": "1",
            "Shipped": "<span class=\"badge badge-light-danger\">Order Cancelled</span>",
            "Total": "$89241",

            "Employee Name": "Jos Buttler",
            "Salary": "$70,000",
            "Skill": "<div class=\"progress-showcase\"><div class=\"progress sm-progress-bar\"><div class=\"progress-bar bg-warning\" role=\"progressbar\" style=\"width: 60%\" aria-valuenow=\"50\" aria-valuemin=\"0\" aria-valuemax=\"100\"></div></div></div>",
            "Office": "Englend",
            "Hours": "4:00",
            "Experience": "8 Year",
        },
        {
            "ID": "L5",
            "Pickup Location": "Sydney, AU",
            "Delivery Location": "Arizona, US",
            "Pickup Date": "26/09/2022",
            "Delivery Date": "12/07/2022",
            "Bid Status": "<span class=\"badge badge-light-success\">Biddable</span>",
            "Price": "$1598.50",
            "Progress": "100%",

            "Id": "5",
            "Product": "i phone 11 pro",
            "Order Id": "#1A84RD3",
            "Quantity": "4",
            "Shipped": "<span class=\"badge badge-light-warning\">On The Way</span>",
            "Total": "$89241",

            "Employee Name": "Otto Clay",
            "Salary": "$10,000",
            "Skill": "<div class=\"progress-showcase\"><div class=\"progress sm-progress-bar\"><div class=\"progress-bar bg-danger\" role=\"progressbar\" style=\"width: 50%\" aria-valuenow=\"50\" aria-valuemin=\"0\" aria-valuemax=\"100\"></div></div></div>",
            "Office": "London",
            "Hours": "9:30",
            "Experience": "5 Year",
        },
        {
            "ID": "L6",
            "Pickup Location": "Sydney, AU",
            "Delivery Location": "Arizona, US",
            "Pickup Date": "26/09/2022",
            "Delivery Date": "26/09/2022",
            "Bid Status": "<span class=\"badge badge-light-danger\">Fixed</span>",
            "Price": "$2315.00",
            "Progress": "75%",

            "Id": "6",
            "Product": "Samsung S22 ultra ",
            "Order Id": "#F8ST59L",
            "Quantity": "5",
            "Shipped": "<span class=\"badge badge-light-danger\">Out For Delivery</span>",
            "Total": "$56893",

            "Employee Name": "Virat Kohli",
            "Salary": "$12,000",
            "Skill": "<div class=\"progress-showcase\"><div class=\"progress sm-progress-bar\"><div class=\"progress-bar bg-success\" role=\"progressbar\" style=\"width: 80%\" aria-valuenow=\"50\" aria-valuemin=\"0\" aria-valuemax=\"100\"></div></div></div>",
            "Office": "India",
            "Hours": "4:30",
            "Experience": "12 Year",
        }
    ];
    db.users = [
        {
            "": "x",
            "Account": "A758A693-0302-03D1-AE53-EEFE22855556",
            "Name": "Carson Kelley",
            "RegisterDate": "2002-04-20T22:55:52-07:00"
        },
        {
            "Account": "D89FF524-1233-0CE7-C9E1-56EFF017A321",
            "Name": "Prescott Griffin",
            "RegisterDate": "2011-02-22T05:59:55-08:00"
        },
        {
            "Account": "06FAAD9A-5114-08F6-D60C-961B2528B4F0",
            "Name": "Amir Saunders",
            "RegisterDate": "2014-08-13T09:17:49-07:00"
        },
        {
            "Account": "EED7653D-7DD9-A722-64A8-36A55ECDBE77",
            "Name": "Derek Thornton",
            "RegisterDate": "2012-02-27T01:31:07-08:00"
        },
        {
            "Account": "2A2E6D40-FEBD-C643-A751-9AB4CAF1E2F6",
            "Name": "Fletcher Romero",
            "RegisterDate": "2010-06-25T15:49:54-07:00"
        },
        {
            "Account": "3978F8FA-DFF0-DA0E-0A5D-EB9D281A3286",
            "Name": "Thaddeus Stein",
            "RegisterDate": "2013-11-10T07:29:41-08:00"
        },
        {
            "Account": "658DBF5A-176E-569A-9273-74FB5F69FA42",
            "Name": "Nash Knapp",
            "RegisterDate": "2005-06-24T09:11:19-07:00"
        },
        {
            "Account": "76D2EE4B-7A73-1212-F6F2-957EF8C1F907",
            "Name": "Quamar Vega",
            "RegisterDate": "2011-04-13T20:06:29-07:00"
        },
        {
            "Account": "00E46809-A595-CE82-C5B4-D1CAEB7E3E58",
            "Name": "Philip Galloway",
            "RegisterDate": "2008-08-21T18:59:38-07:00"
        },
        {
            "Account": "C196781C-DDCC-AF83-DDC2-CA3E851A47A0",
            "Name": "Mason French",
            "RegisterDate": "2000-11-15T00:38:37-08:00"
        },
        {
            "Account": "5911F201-818A-B393-5888-13157CE0D63F",
            "Name": "Ross Cortez",
            "RegisterDate": "2010-05-27T17:35:32-07:00"
        },
        {
            "Account": "B8BB78F9-E1A1-A956-086F-E12B6FE168B6",
            "Name": "Logan King",
            "RegisterDate": "2003-07-08T16:58:06-07:00"
        },
        {
            "Account": "06F636C3-9599-1A2D-5FD5-86B24ADDE626",
            "Name": "Cedric Leblanc",
            "RegisterDate": "2011-06-30T14:30:10-07:00"
        },
        {
            "Account": "FE880CDD-F6E7-75CB-743C-64C6DE192412",
            "Name": "Simon Sullivan",
            "RegisterDate": "2013-06-11T16:35:07-07:00"
        },
        {
            "Account": "BBEDD673-E2C1-4872-A5D3-C4EBD4BE0A12",
            "Name": "Jamal West",
            "RegisterDate": "2001-03-16T20:18:29-08:00"
        },
        {
            "Account": "19BC22FA-C52E-0CC6-9552-10365C755FAC",
            "Name": "Hector Morales",
            "RegisterDate": "2012-11-01T01:56:34-07:00"
        },
        {
            "Account": "A8292214-2C13-5989-3419-6B83DD637D6C",
            "Name": "Herrod Hart",
            "RegisterDate": "2008-03-13T19:21:04-07:00"
        },
        {
            "Account": "0285564B-F447-0E7F-EAA1-7FB8F9C453C8",
            "Name": "Clark Maxwell",
            "RegisterDate": "2004-08-05T08:22:24-07:00"
        },
        {
            "Account": "EA78F076-4F6E-4228-268C-1F51272498AE",
            "Name": "Reuben Walter",
            "RegisterDate": "2011-01-23T01:55:59-08:00"
        },
        {
            "Account": "6A88C194-EA21-426F-4FE2-F2AE33F51793",
            "Name": "Ira Ingram",
            "RegisterDate": "2008-08-15T05:57:46-07:00"
        },
        {
            "Account": "4275E873-439C-AD26-56B3-8715E336508E",
            "Name": "Damian Morrow",
            "RegisterDate": "2015-09-13T01:50:55-07:00"
        },
        {
            "Account": "A0D733C4-9070-B8D6-4387-D44F0BA515BE",
            "Name": "Macon Farrell",
            "RegisterDate": "2011-03-14T05:41:40-07:00"
        },
        {
            "Account": "B3683DE8-C2FA-7CA0-A8A6-8FA7E954F90A",
            "Name": "Joel Galloway",
            "RegisterDate": "2003-02-03T04:19:01-08:00"
        },
        {
            "Account": "01D95A8E-91BC-2050-F5D0-4437AAFFD11F",
            "Name": "Rigel Horton",
            "RegisterDate": "2015-06-20T11:53:11-07:00"
        },
        {
            "Account": "F0D12CC0-31AC-A82E-FD73-EEEFDBD21A36",
            "Name": "Sylvester Gaines",
            "RegisterDate": "2004-03-12T09:57:13-08:00"
        },
        {
            "Account": "874FCC49-9A61-71BC-2F4E-2CE88348AD7B",
            "Name": "Abbot Mckay",
            "RegisterDate": "2008-12-26T20:42:57-08:00"
        },
        {
            "Account": "B8DA1912-20A0-FB6E-0031-5F88FD63EF90",
            "Name": "Solomon Green",
            "RegisterDate": "2013-09-04T01:44:47-07:00"
        }
     ];
}());