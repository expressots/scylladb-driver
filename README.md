Scylla Nodejs Driver
====================

A modern, fully feature scylladb driver for Nodejs developed using Node Native API.

The driver supports Node versions 18.X and above.

Features
--------

* `Synchronous`_ and `Asynchronous operations` - Sync and async APIs
* `Connecting to the cluster` - Configuring a connection to scylladb cluster
* `Creating and executing queries` - Simple queries, query values, query results, prepared statements, batch statements, paging, keyspace switching, schema agreement, lightweight transactions, query timeouts
* `Profiles` - Creating, configuring and using profiles, profile policies, priorities of execution settings, remapping execution profiles handling
* `Data types` - Handling all scylla supported data types, custom data types, user defined types
* `Load balancing policy` - Defining and configuring the load balancing policy
* `Retries policy` - Defining retry policies `DefaultRetryPolicy`, `DowngradingConsistencyRetryPolicy`
* `Speculative execution` - Optimizing query either using `Simple` or `LatencyAware` speculative execution policies
* `Metrics` - Enable, configure and collect metrics
* `Logging` - Enable logging
* `Query tracing` - Enable query tracing for queries
* `Schema` - Fetch, inspect database schema
 
Installation
------------

Installation through npm is recommended::

    $ npm i scylladb-driver

For more complete installation instructions, see the official documentation at `installation guide <http://>`_.

Documentation
-------------

The official documentation can be found `here <http://>`_.

Information includes:

* `How to instal and use scylladb driver <http://>`_
* `Getting started guide <http://>`_
* `API docs <http://python-driver.docs.scylladb.com/stable/api/index.html>`_
* `Performance tips <http://>`_
* `Development roadmap <http://>`_

Contributing
------------

See `CONTRIBUTING <https://>`_.

Report Issues
------------------

Please report any bugs and make any feature requests by clicking the New Issue button in 
`Github <https://github.com/scylladb/python-driver/issues>`_.

If you would like to contribute, please feel free to send a pull request.

Getting Support
-----------------

Your best chances for getting quick support is either open an issue or ask a question in the discord channel `Scylla Discord <http://>`_.

License
-------

