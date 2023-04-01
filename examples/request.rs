//! This example demonstrates making an xmlrpc request using the reqwest libary as a
//! transport.
//! 
//! This example is intended to target a ROS1 master node, but is include more for
//! illustrative purposes of showing how this library can be used than as a practical
//! example.
fn main() {
    let my_id = "xmlrpc_example"; // Used as an argument to ROS server
    let server_uri = "http://localhost:11311"; // Default location where ROS master runs its xmlrpc server
    let client = reqwest::blocking::Client::new(); // Create our client

    // Use this library to generate the body of the http request
    let body = serde_xmlrpc::request_to_string("getTopicTypes", vec![my_id.into()]).unwrap();

    // Send our request to the server and get a response back (using blocking API for simplicity)
    let response = client.post(server_uri).body(body).send().unwrap().text().unwrap();
    
    // Use this libaray to parse the values back out of the response
    // The ROS master server will give us 3 values back:
    // a status code to indicate success
    // an error message if anything went wrong
    // and a list of tuples of (topic name, topic data type)
    let (_status_code, _error_msg, topics) = serde_xmlrpc::response_from_str::<(i8, String, Vec<(String, String)>)>(&response).unwrap();

    println!("Ros reported the following registered topics and types: {topics:?}");

    // If every thing worked and you had a running ROS master on your system you should get something like the following:
    // Ros reported the following registered topics and types: [("/rosout_agg", "rosgraph_msgs/Log"), ("/rosout", "rosgraph_msgs/Log")]
}